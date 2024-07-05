use std::{fs, net::{SocketAddr, IpAddr, Ipv4Addr}};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::sync::{Arc, Mutex};

use log::{error, info};

use openssl::pkey::Private;
use openssl::rsa::Rsa;

use tokio::io;
use tokio::net::{TcpListener, TcpStream};

use osp_protocol::OSPUrl;

use crate::connection::inbound::{InboundConnection, TransferState};
use crate::connection::outbound::OutboundConnection;

pub struct InitState {
    private_key: Option<Rsa<Private>>,
}

pub struct ConnectionState {
    private_key: Rsa<Private>,
}

#[derive(Clone)]
pub struct OSProtocolNode<TState> {
    bind_addr: SocketAddr,
    hostname: String,
    state: Arc<Mutex<TState>>,
}

impl OSProtocolNode<InitState> {
    pub fn new() -> Self {
        OSProtocolNode::<InitState> {
            bind_addr: SocketAddr::new(IpAddr::from(Ipv4Addr::LOCALHOST), 57401),
            hostname: "".to_string(),
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

    pub fn set_private_key_file(&mut self, path: String) {
        let key_contents = fs::read_to_string(path.clone()).expect(format!("Unable to open private key file {}", path).as_str());
        self.state.lock().unwrap().private_key = Some(Rsa::private_key_from_pem(key_contents.as_bytes()).unwrap());
    }

    pub fn init(&mut self) -> OSProtocolNode<ConnectionState> {
        let bind_addr = self.bind_addr.clone();
        let hostname = self.hostname.clone();
        let private_key = self.state.lock().unwrap().private_key.clone().unwrap();
        OSProtocolNode::<ConnectionState> {
            bind_addr,
            hostname,
            state: Arc::new(Mutex::new(ConnectionState {
                private_key,
            })),
        }
    }
}

impl OSProtocolNode<ConnectionState> {
    pub async fn listen<'a, F, Fut>(&'a mut self, conn_handler: F) -> io::Result<()>
    where
        F: Fn(InboundConnection<TransferState>, &Arc<Mutex<ConnectionState>>) -> Fut + Send + Copy + 'static,
        Fut: Future<Output = Result<(), ()>> + Send + 'static,
    {
        let port = self.bind_addr.port();
        let listener = TcpListener::bind(self.bind_addr).await?;
        info!("Listening started on port {port}, ready to accept connections");

        loop {
            // The second item contains the IP and port of the new connection.
            let (stream, _) = listener.accept().await?;

            info!(
                "Accepting a new connection from {}",
                stream
                    .peer_addr()
                    .map(|addr| addr.to_string())
                    .unwrap_or("unknown address".to_string())
            );

            let state_rc = self.state.clone();
            tokio::spawn(async move {
                let mut connection_handshake = InboundConnection::with_stream(stream).unwrap();
                match connection_handshake.begin().await {
                    Ok(_) => {
                        let connection_transfer = InboundConnection::<TransferState>::from(connection_handshake);

                        match conn_handler(connection_transfer, &state_rc).await {
                            Ok(_) => {}
                            Err(_) => {}
                        }
                    }
                    Err(e) => {
                        error!("Handshake failed: {e}");
                    }
                }
            });
        }
    }

    pub async fn create_outbound(&self, url: OSPUrl) -> io::Result<()> {
        info!("Starting outbound connection to {url}");
        let mut conn = OutboundConnection::create(
            url,
            self.state.lock().unwrap().private_key.clone(),
            self.hostname.clone()
        ).await?;
        let mut conn_in_handshake = conn.begin().await?;
        conn_in_handshake.handshake().await
    }
}
