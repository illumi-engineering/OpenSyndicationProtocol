use std::io;
use std::net::TcpStream;
use osp_protocol::{ConnectionType, OSPHandshakeIn, OSPHandshakeOut, Protocol};
use crate::OSProtocolNode;

pub struct InboundConnection<'a> {
    protocol: Protocol,
    node: &'a OSProtocolNode<'a>,
    connection_type: ConnectionType,
}

impl<'a> InboundConnection<'a> {
    pub fn with_stream(node: &'a OSProtocolNode, stream: TcpStream) -> io::Result<Self> {
        Ok(Self {
            protocol: Protocol::with_stream(stream)?,
            node,
            connection_type: ConnectionType::Unknown,
        })
    }

    pub fn begin(mut self) -> io::Result<()> {
        let request = self.protocol.read_message::<OSPHandshakeIn>()?;
        match request {
            OSPHandshakeIn::Hello { connection_type } => {
                self.connection_type = connection_type;
                self.protocol.send_message(&OSPHandshakeOut::Acknowledge {
                    ok: true,
                    require_login: false,
                    err: None
                })?
            }
            OSPHandshakeIn::Login { } => {
                // todo: check server configuration
            }
        }

        Ok(())
    }
}