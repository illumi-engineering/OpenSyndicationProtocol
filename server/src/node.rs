use std::{net::{SocketAddr, TcpListener, TcpStream}};
use std::net::{IpAddr, Ipv4Addr};
use osp_protocol::{ConnectionType, Protocol};
use crate::connection::inbound::{InboundConnection, InboundConnectionOptions};
use crate::connection::outbound::OutboundConnection;

struct InboundConnectionContext {
    protocol: Protocol,
    connection_type: ConnectionType,
}

struct OutboundConnectionContext {
    protocol: Protocol
}

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

    pub fn require_auth(mut self, client: bool, server: bool) -> Self {
        self.require_client_auth = client;
        self.require_server_auth = server;
        self
    }

    pub fn build(self) -> OSProtocolNode {
        OSProtocolNode {
            // inbound_conn_id: Arc::new(RwLock::new(0)),
            // outbound_conn_id: Arc::new(RwLock::new(0)),
            bind_addr: self.bind_addr,
            // inbound_connections: Arc::new(Mutex::new(HashMap::new())),
            // outbound_connections: Arc::new(Mutex::new(HashMap::new())),
            require_client_auth: self.require_client_auth,
            require_server_auth: self.require_server_auth,
        }
    }
}

#[derive(Clone)]
pub struct OSProtocolNode {
    // inbound_conn_id: Arc<RwLock<u32>>,
    // outbound_conn_id: Arc<RwLock<u32>>,
    bind_addr: SocketAddr,
    // inbound_connections: Arc<Mutex<HashMap<u32, InboundConnection>>>,
    // outbound_connections: Arc<Mutex<HashMap<u32, OutboundConnection>>>,
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

                self.clone().start_connection(stream);
            }
        }
    }

    fn start_connection(self, stream: TcpStream) {
        // let inbound_conn_id = self.inbound_conn_id.clone();
        // let inbound_connections = self.inbound_connections.clone();

        std::thread::spawn(move | | {
            let options = InboundConnectionOptions::build()
                .require_auth(self.require_client_auth, self.require_server_auth);
            let connection = InboundConnection::with_stream(stream, options).unwrap();
            connection.begin()

            //
            // let id = inbound_conn_id.read().unwrap();
            // inbound_connections.lock().unwrap().insert(*id, connection);
            // let new_id = id.clone();
            //
            // let mut next_id = inbound_conn_id.write().unwrap();
            // *next_id += 1;
            //
            // move | | {
            //     let mut conns = inbound_connections.lock().unwrap();
            //     let conn = conns.get(&new_id).unwrap();
            // }
        });
    }

    pub fn start_syndication_from(self, addr: SocketAddr) {
        // let outbound_conn_id = self.outbound_conn_id.clone();
        // let outbound_connections = self.outbound_connections.clone();

        std::thread::spawn(move | | {
            let protocol = Protocol::connect(addr).unwrap();
            let mut connection = OutboundConnection::create(protocol);
            connection.begin()
        });
    }
}