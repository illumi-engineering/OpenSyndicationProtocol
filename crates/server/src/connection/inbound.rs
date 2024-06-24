use log::info;
use openssl::rand::rand_bytes;
use openssl::rsa::{Padding, Rsa};
use tokio::io;
use tokio::net::TcpStream;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::Resolver;
use uuid::Uuid;
use osp_protocol::{Protocol};
use osp_protocol::;

pub struct InboundConnection<TState> {
    connection_type: ConnectionType,
    state: TState
}

pub struct HandshakeState {
    nonce: Uuid,
}
pub struct TransferState {

}

impl From<InboundConnection<HandshakeState>> for InboundConnection<TransferState> {
    fn from(value: InboundConnection<HandshakeState>) -> Self {
        InboundConnection {
            protocol: value.protocol,
            connection_type: value.connection_type,
            state: TransferState {},
        }
    }
}

impl InboundConnection<HandshakeState> {
    pub fn with_stream(stream: TcpStream) -> io::Result<Self> {
        Ok(Self {
            protocol: Protocol::with_stream(stream)?,
            connection_type: ConnectionType::Unknown,
            state: HandshakeState {
                nonce: Uuid::new_v4(),
            }
        })
    }

    async fn send_close_err(&mut self, error_kind: io::ErrorKind, err: String) -> io::Error {
        self.protocol.send_message(&OSPHandshakeOut::Close {
            can_continue: false,
            err: Some(err.clone()),
        }).await.unwrap();
        io::Error::new(error_kind, err)
    }

    pub async fn begin(&mut self) -> io::Result<()> {
        if let OSPHandshakeIn::Hello { connection_type } = self.protocol.read_frame::<OSPHandshakeIn>().await? {
            self.connection_type = connection_type;

            self.protocol.send_message(&OSPHandshakeOut::Acknowledge {
                ok: true,
                err: None
            }).await?;

            Ok(())

        //     if let OSPHandshakeIn::Identify { hostname } = self.protocol.read_message::<OSPHandshakeIn>()? {
        //         // todo: check whitelist/blacklist
        //         info!("Looking up challenge record for {hostname}");
        //         let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
        //         let txt_resp = resolver.txt_lookup(format!("_osp.{}", hostname));
        //         match txt_resp {
        //             Ok(txt_resp) => {
        //                 if let Some(record) = txt_resp.iter().next() {
        //                     info!("Challenge record found");
        //                     let pub_key = Rsa::public_key_from_pem(record.to_string().as_bytes())?;
        //                     let mut challenge_bytes = [0; 256];
        //                     rand_bytes(&mut challenge_bytes).unwrap();
        //                     let mut encrypted_challenge = vec![0u8; pub_key.size() as usize];
        //                     pub_key.public_encrypt(&challenge_bytes, &mut encrypted_challenge, Padding::PKCS1)?;
        //                     self.protocol.send_message(&OSPHandshakeOut::Challenge {
        //                         encrypted_challenge,
        //                         nonce: self.state.nonce,
        //                     })?;
        //
        //                     if let OSPHandshakeIn::Verify { challenge, nonce } = self.protocol.read_message::<OSPHandshakeIn>()? {
        //                         info!("Received challenge verification");
        //                         if nonce != self.state.nonce {
        //                             return Err(self.send_close_err(io::ErrorKind::InvalidData, "Invalid nonce".to_string()));
        //                         }
        //
        //                         if challenge == challenge_bytes {
        //                             info!("Challenge verification successful");
        //                             self.protocol.send_message(&OSPHandshakeOut::Close {
        //                                 can_continue: true,
        //                                 err: None,
        //                             })?;
        //                             Ok(())
        //                         } else {
        //                             return Err(self.send_close_err(io::ErrorKind::PermissionDenied, "Challenge failed".to_string()))
        //                         }
        //                     } else {
        //                         return Err(self.send_close_err(io::ErrorKind::InvalidInput, "Expected challenge verification packet".to_string()));
        //                     }
        //                 } else {
        //                     return Err(
        //                         self.send_close_err(
        //                             io::ErrorKind::InvalidData,
        //                             format!("Failed to resolve SRV record for {}. Is it located at _osp.{}?", hostname, hostname)
        //                         )
        //                     );
        //                 }
        //             }
        //             Err(e) => {
        //                 return Err(
        //                     self.send_close_err(
        //                         io::ErrorKind::Other,
        //                         format!(
        //                             "Failed to resolve SRV record for {}. Is it located at _osp.{}?\n\nFurther Details: {}",
        //                             hostname, hostname, e.to_string()
        //                         )
        //                     )
        //                 );
        //             }
        //         }
        //     } else {
        //         return Err(self.send_close_err(io::ErrorKind::InvalidInput, "Expected identify packet".to_string()));
        //     }
        } else {
            return Err(self.send_close_err(io::ErrorKind::InvalidInput, "Expected hello packet".to_string()).await);
        }
    }
}