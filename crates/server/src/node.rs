use std::{fs, net::{SocketAddr, IpAddr, Ipv4Addr}};
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use bincode::Encode;

use log::{error, info, warn};

use openssl::pkey::Private;
use openssl::rsa::Rsa;

use tokio::io;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;
use osp_data::{Data};

use osp_data::registry::DataTypeRegistry;
use osp_protocol::OSPUrl;
use osp_protocol::utils::ConnectionIntent;

use crate::connection::inbound::{InboundConnection, TransferState as InboundTransferState};
use crate::connection::outbound::{HandshakeState as OutboundHandshakeState, OutboundConnection};

pub struct InitState {
    private_key: Option<Rsa<Private>>,
}

#[derive(Clone)]
pub struct ConnectionState {
    private_key: Rsa<Private>,
    subscribed_hostnames: Vec<String>,
}

#[derive(Clone)]
pub struct Node<TState> {
    bind_addr: SocketAddr,
    hostname: String,
    data_types: Arc<Mutex<DataTypeRegistry>>,
    state: Arc<Mutex<TState>>,
}

impl Node<InitState> {
    #[must_use] pub fn new() -> Self {
        Self {
            bind_addr: SocketAddr::new(IpAddr::from(Ipv4Addr::LOCALHOST), 57401),
            hostname: String::new(),
            data_types: Arc::new(Mutex::new(DataTypeRegistry::new())),
            state: Arc::new(Mutex::new(InitState {
                private_key: None,
            })),
        }
    }

    pub fn set_addr(&mut self, addr: SocketAddr) {
        self.bind_addr = addr;
    }

    pub fn set_hostname(&mut self, hostname: String) {
        self.hostname = hostname;
    }

    /// Set the node's private key from a file path
    /// 
    /// # Errors
    /// - If reading the file at `path` fails
    /// - [`openssl::error::ErrorStack`] If constructing the private key from file fails
    pub async fn set_private_key_file(&self, path: String) -> io::Result<()> {
        let key_contents = fs::read_to_string(path.clone())?;
        self.state.lock().await.private_key = Some(Rsa::private_key_from_pem(key_contents.as_bytes())?);
        Ok(())
    }

    /// Register a new [`DataType`] for `TData` to be handled by this node.
    /// 
    /// 
    pub async fn register_data_type<TData>(&self)
    where
        TData : Data + 'static,
    {
        self.data_types.lock().await.register::<TData>();
    }

    /// Initialize the node and enter [`ConnectionState`].
    /// 
    /// # Panics
    /// - If private key is not set
    pub async fn init(&self) -> Node<ConnectionState> {
        let bind_addr = self.bind_addr;
        let hostname = self.hostname.clone();
        let private_key = self.state.lock().await.private_key.clone().expect("Private key must be set!");
        Node::<ConnectionState> {
            bind_addr,
            hostname,
            data_types: self.data_types.clone(),
            state: Arc::new(Mutex::new(ConnectionState {
                private_key,
                subscribed_hostnames: Vec::new(),
            })),
        }
    }
}

impl Default for Node<InitState> {
    fn default() -> Self {
        Self::new()
    }
}

impl Node<ConnectionState> {
    pub async fn listen(&self) -> io::Result<()> {
        let port = self.bind_addr.port();
        let listener = TcpListener::bind(self.bind_addr).await?;
        info!("Listening started on port {port}, ready to accept connections");

        loop {
            // The second item contains the IP and port of the new connection.
            let (stream, _) = listener.accept().await?;

            info!(
                "Accepting a new connection from {}",
                stream
                    .peer_addr().map_or_else(|_| "unknown address".to_string(), |addr| addr.to_string())
            );

            let state_rc = self.state.clone();
            let data_types_rc = self.data_types.clone();
            tokio::spawn(async move {
                let mut connection_handshake = InboundConnection::with_stream(stream, data_types_rc);
                match connection_handshake.begin().await {
                    Ok((intent, hostname)) => {
                        match intent {
                            ConnectionIntent::Subscribe => {
                                state_rc.lock().await.subscribed_hostnames.push(hostname);
                            }
                            ConnectionIntent::TransferData => {
                                let _connection_transfer = connection_handshake.start_transfer();
                            }
                            ConnectionIntent::Unknown => {
                                warn!("Unknown connection intent. Closing...");
                            }
                        }
                        // match conn_handler(connection_transfer, &state_rc).await {
                        //     Ok(_) => {}
                        //     Err(_) => {}
                        // }
                    }
                    Err(e) => {
                        error!("Handshake failed: {e}");
                    }
                }
            });
        }
    }

    async fn resolve_osp_url_from_srv(hostname: String) -> io::Result<OSPUrl> {
        let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

        let srv_resp = resolver.srv_lookup(format!("_osp.{hostname}")).await?;
        srv_resp.iter().next().map_or_else(|| Err(Error::new(ErrorKind::HostUnreachable, format!("Unable to lookup srv record at _osp.{hostname}"))), |srv| {
            let port = srv.port();
            let domain = srv.target().to_string();

            Ok(OSPUrl {
                domain,
                port,
            })
        })
    }

    async fn create_outbound(&self, url: OSPUrl) -> io::Result<OutboundConnection<OutboundHandshakeState>> {
        info!("Starting outbound connection to {url}");
        let mut conn = OutboundConnection::create(
            url,
            self.state.lock().await.private_key.clone(),
            self.hostname.clone(),
            self.data_types.clone(),
        ).await?;
        let mut conn_in_handshake = conn.begin().await?;
        conn_in_handshake.handshake().await?;
        Ok(conn_in_handshake)
    }

    pub async fn broadcast_data<TData>(&self, obj: TData) -> io::Result<()>
    where
        TData : Data + 'static + Clone + Encode
    {
        let hostnames = &mut self.state.lock().await.subscribed_hostnames;
        for hostname in hostnames {
            let outbound = self.create_outbound(Self::resolve_osp_url_from_srv(hostname.clone()).await?).await?;
            let mut outbound_transfer = outbound.start_transfer().await?;
            outbound_transfer.send_data(obj.clone()).await?;
        }

        Ok(())
    }

    /// Subscribe to updates from an external [`Node`] at `url`.
    /// 
    /// # Errors
    /// - Delegated from [`self.create_outbound`] and [`OutboundConnection<OutboundHandshakeState>::subscribe`]
    pub async fn subscribe_to(self, url: OSPUrl) -> io::Result<()> {
        let mut outbound = self.create_outbound(url).await?;
        outbound.subscribe().await
    }
}
