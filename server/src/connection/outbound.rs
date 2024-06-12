use std::io;
use osp_protocol::{ConnectionType, OSPHandshakeIn, OSPHandshakeOut, Protocol};

pub struct OutboundConnection {
    protocol: Protocol
}

impl OutboundConnection {
    pub fn create(protocol: Protocol) -> Self {
        Self {
            protocol
        }
    }

    pub fn begin(mut self) -> io::Result<()> {
        self.protocol.send_message(&OSPHandshakeIn::Hello { connection_type: ConnectionType::Server })?;

        if let OSPHandshakeOut::Acknowledge {
            ok,
            require_login,
            err
        } = self.protocol.read_message::<OSPHandshakeOut>()? {
            if ok {
                // todo: next steps
                if require_login {

                }
            } else {
                eprintln!(format!("[osp_server:outbound] hello: {}", err.unwrap()))
            }
        }

        Ok(())
    }
}