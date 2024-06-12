use std::io;
use std::net::TcpStream;
use osp_protocol::{ConnectionType, OSPHandshakeIn, OSPHandshakeOut, Protocol};

pub struct InboundConnectionOptions {
    require_client_auth: bool,
    require_server_auth: bool,
}

impl InboundConnectionOptions {
    pub fn build() -> Self {
        Self {
            require_client_auth: false,
            require_server_auth: false,
        }
    }

    pub fn require_auth(mut self, client: bool, server: bool) -> Self {
        self.require_client_auth = client;
        self.require_server_auth = server;
        self
    }
}

pub struct InboundConnection {
    protocol: Protocol,
    connection_type: ConnectionType,
    options: InboundConnectionOptions,
}

impl InboundConnection {
    pub fn with_stream(stream: TcpStream, options: InboundConnectionOptions) -> io::Result<Self> {
        Ok(Self {
            protocol: Protocol::with_stream(stream)?,
            connection_type: ConnectionType::Unknown,
            options,
        })
    }

    pub fn begin(mut self) -> io::Result<()> {
        let request = self.protocol.read_message::<OSPHandshakeIn>()?;
        match request {
            OSPHandshakeIn::Hello { connection_type } => {
                self.connection_type = connection_type;

                let require_login= match self.connection_type {
                    ConnectionType::Client => self.options.require_client_auth,
                    ConnectionType::Server => self.options.require_server_auth,
                    _ => true
                };

                self.protocol.send_message(&OSPHandshakeOut::Acknowledge {
                    ok: true,
                    require_login,
                    err: None
                })?
            }
            OSPHandshakeIn::Login { } => {
                // todo: login logic
            }
        }

        Ok(())
    }
}