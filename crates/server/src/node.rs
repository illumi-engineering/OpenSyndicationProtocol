use std::{fs, io, net::{SocketAddr, TcpListener, TcpStream}};
use std::net::{IpAddr, Ipv4Addr};
use log::info;
use openssl::pkey::Private;
use openssl::rsa::Rsa;
use osp_protocol::OSPUrl;
use crate::connection::inbound::{InboundConnection, TransferState};
use crate::connection::outbound::OutboundConnection;


pub struct OSProtocolNodeBuilder {
    bind_addr: SocketAddr,
    hostname: String,
    private_key: Option<Rsa<Private>>,
}

impl OSProtocolNodeBuilder {
    pub fn bind_to(mut self, addr: SocketAddr) -> Self {
        self.bind_addr = addr;
        self
    }

    pub fn hostname(mut self, hostname: String) -> Self {
        self.hostname = hostname;
        self
    }

    pub fn private_key_file(mut self, path: String) -> Self {
        let key_contents = fs::read_to_string(path.clone()).expect(format!("Unable to open private key file {}", path).as_str());
        self.private_key = Some(Rsa::private_key_from_pem(key_contents.as_bytes()).unwrap());
        self
    }

    pub fn build(self) -> OSProtocolNode {
        OSProtocolNode {
            bind_addr: self.bind_addr,
            hostname: self.hostname,
            private_key: self.private_key.unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct OSProtocolNode {
    bind_addr: SocketAddr,
    hostname: String,
    private_key: Rsa<Private>,
}

impl OSProtocolNode {
    pub fn builder() -> OSProtocolNodeBuilder {
        OSProtocolNodeBuilder {
            bind_addr: SocketAddr::new(IpAddr::from(Ipv4Addr::LOCALHOST), 57401),
            hostname: "".to_string(),
            private_key: None,
        }
    }

    pub fn listen(&self) {
        let port = self.bind_addr.port();
        let listener = TcpListener::bind(self.bind_addr).unwrap();
        info!("Listening started on port {port}, ready to accept connections");
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                info!("Accepting a new connection from {}",
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
        std::thread::spawn(move | | {
            let mut connection_handshake = InboundConnection::with_stream(stream).unwrap();
            match connection_handshake.begin() {
                Ok(_) => {
                    let mut connection_transfer = InboundConnection::<TransferState>::from(connection_handshake);
                }
                _ => {}
            }
        });
    }

    pub fn test_outbound(&self, url: OSPUrl) {
        info!("Testing outbound connection to {url}");
        let mut conn = OutboundConnection::create(url, self.private_key.clone(), self.hostname.clone()).unwrap();
        conn.begin().unwrap();
    }
}