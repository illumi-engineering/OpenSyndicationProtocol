use std::{net::{SocketAddr, TcpListener, TcpStream}};
use std::net::{IpAddr, Ipv4Addr};
use crate::connection::inbound::{InboundConnection, TransferState};


struct OSProtocolNodeBuilder {
    bind_addr: SocketAddr,
}

impl OSProtocolNodeBuilder {
    pub fn bind_to(mut self, addr: SocketAddr) -> Self {
        self.bind_addr = addr;
        self
    }

    pub fn build(self) -> OSProtocolNode {
        OSProtocolNode {
            bind_addr: self.bind_addr,
        }
    }
}

#[derive(Clone)]
pub struct OSProtocolNode {
    bind_addr: SocketAddr,
}

impl OSProtocolNode {
    fn builder() -> OSProtocolNodeBuilder {
        OSProtocolNodeBuilder {
            bind_addr: SocketAddr::new(IpAddr::from(Ipv4Addr::LOCALHOST), 57401),
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
}