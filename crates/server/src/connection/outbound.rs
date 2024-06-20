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

    pub fn begin(&mut self) -> io::Result<()> {
        self.protocol.send_message(&OSPHandshakeIn::Hello { connection_type: ConnectionType::Server })?;

        if let OSPHandshakeOut::Acknowledge {
            ok,
            err
        } = self.protocol.read_message::<OSPHandshakeOut>()? {
            if ok {
            } else {
                eprintln!("[osp_server:outbound] hello: {}", err.unwrap())
            }
        }

        Ok(())
    }
}