use std::{collections::HashMap, net::{SocketAddr, TcpListener, TcpStream}, sync::{Arc, Mutex, RwLock}};
use std::net::{IpAddr, Ipv4Addr};
use osp_protocol::{ConnectionType, Protocol};
use crate::connection::inbound::InboundConnection;
use crate::connection::outbound::OutboundConnection;

struct InboundConnectionContext {
    protocol: Protocol,
    connection_type: ConnectionType,
}

struct OutboundConnectionContext {
    protocol: Protocol
}

#[derive(Default)]
struct OSProtocolNodeBuilder {
    bind_addr: SocketAddr,
    require_server_auth: bool,
    require_client_auth: bool,
}

impl OSProtocolNodeBuilder {
    pub fn bind_to(mut self, addr: SocketAddr) -> Self {
        self.bind_addr = addr;
        self
    }

    pub fn require_server_auth(mut self, auth: bool) -> Self {
        self.require_server_auth = auth;
        self
    }

    pub fn require_client_auth(mut self, auth: bool) -> Self {
        self.require_client_auth = auth;
        self
    }

    pub fn build() -> OSProtocolNode {
        OSProtocolNode {
            inbound_conn_id: Arc::new(RwLock::new(0)),
            outbound_conn_id: Arc::new(RwLock::new(0)),
            bind_addr,
            inbound_connections: Arc::new(Mutex::new(HashMap::new())),
            outbound_connections: Arc::new(Mutex::new(HashMap::new())),
            require_client_auth,
            require_server_auth,
        }
    }
}

#[derive(Clone)]
pub struct OSProtocolNode<'a> {
    inbound_conn_id: Arc<RwLock<u32>>,
    outbound_conn_id: Arc<RwLock<u32>>,
    bind_addr: SocketAddr,
    inbound_connections: Arc<Mutex<HashMap<u32, InboundConnection<'a>>>>,
    outbound_connections: Arc<Mutex<HashMap<u32, OutboundConnection>>>,
    require_server_auth: bool,
    require_client_auth: bool,
}

impl OSProtocolNode {
    // fn create(bind_addr: SocketAddr) -> Self {
    //     Self {
    //         inbound_conn_id: Arc::new(RwLock::new(0)),
    //         outbound_conn_id: Arc::new(RwLock::new(0)),
    //         bind_addr,
    //         inbound_connections: Arc::new(Mutex::new(HashMap::new())),
    //         outbound_connections: Arc::new(Mutex::new(HashMap::new()))
    //     }
    // }

    fn builder() -> OSProtocolNodeBuilder {
        OSProtocolNodeBuilder {
            bind_addr: SocketAddr::new(IpAddr::from(Ipv4Addr::LOCALHOST), 57401),
            require_server_auth: false,
            require_client_auth: false,
        }
    }

    fn start_server(self) {
        let listener = TcpListener::bind(self.bind_addr).unwrap();
        println!("listening started, ready to accept");
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                println!("Accepting a new connection from {}",
                         stream
                             .peer_addr()
                             .map(|addr| addr.to_string())
                             .unwrap_or("unknown address".to_string())
                );

                self.clone().start_connection(stream).map_err(|e| eprintln!("[osp] err: {}", e)).unwrap();
            }
        }
    }

    fn start_connection(&self, stream: TcpStream) {
        let me = self.clone();

        std::thread::spawn({
            let conn = InboundConnection::with_stream(self, stream)?;

            let id = me.inbound_conn_id.read().unwrap();
            me.inbound_connections.lock().unwrap().insert(*id, conn);
            let new_id = id.clone();

            let mut next_id = self.inbound_conn_id.write().unwrap();
            *next_id += 1;

            move || {
                let mut conn = me.inbound_connections.lock()?.get_mut(&new_id)?;
                conn.begin()
            }
        });
    }

    pub fn start_syndication_from(self, addr: SocketAddr) {
        let me = self.clone();

        std::thread::spawn({
            let protocol = Protocol::connect(addr)?;
            let mut conn = OutboundConnection::create(protocol);

            let id = me.outbound_conn_id.read().unwrap();
            me.outbound_connections.lock().unwrap().insert(*id, conn);

            let mut next_id = self.outbound_conn_id.write().unwrap();
            *next_id += 1;

            move || {
                conn.begin()
            }
        });
    }
}