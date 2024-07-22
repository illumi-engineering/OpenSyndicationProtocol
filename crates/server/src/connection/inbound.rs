use std::io::Error;
use std::sync::Arc;
use log::{debug, error, info, warn};

use openssl::rand::rand_bytes;
use openssl::rsa::{Padding, Rsa};

use tokio::io;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use trust_dns_resolver::{TokioAsyncResolver};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};

use uuid::Uuid;
use osp_data::registry::DataTypeRegistry;

use osp_protocol::{ConnectionType, Protocol};
use osp_protocol::packet::{PacketDecoder, PacketEncoder};
use osp_protocol::packet::handshake::{HandshakePacketGuestToHost, HandshakePacketHostToGuest};
use osp_protocol::packet::transfer::{TransferPacketHostToGuest, TransferPacketGuestToHost};
use osp_protocol::utils::ConnectionIntent;

#[allow(clippy::module_name_repetitions)]
pub struct InboundConnection<TState> {
    connection_type: ConnectionType,
    data_types: Arc<Mutex<DataTypeRegistry>>,
    state: TState
}

// Satoru Gojo is best boi

pub struct HandshakeState {
    nonce: Uuid,
    protocol: Protocol<HandshakePacketGuestToHost, HandshakePacketHostToGuest>
}
pub struct TransferState {
    protocol: Protocol<TransferPacketGuestToHost, TransferPacketHostToGuest>
}

impl InboundConnection<HandshakeState> {
    pub fn with_stream(stream: TcpStream, data_types: Arc<Mutex<DataTypeRegistry>>) -> Self {
        Self {
            connection_type: ConnectionType::Unknown,
            data_types,
            state: HandshakeState {
                nonce: Uuid::new_v4(),
                protocol: Protocol::with_stream(stream),
            }
        }
    }

    async fn send_close_err(&mut self, error_kind: io::ErrorKind, err: String) -> Error {
        error!("Closing connection with error: {}", err.clone());
        self.state.protocol.send_message(HandshakePacketHostToGuest::Close {
            can_continue: false,
            err: Some(err.clone()),
        }).await.expect("Failed to send error to guest.");
        Error::new(error_kind, err)
    }


    /// Begin handling an inbound connection
    /// 
    /// # Errors
    /// Returns [`io::ErrorKind::InvalidData`] if:
    /// - Guest hostname does not have a valid TXT record associated for challenge
    /// 
    /// Returns [`io::ErrorKind::InvalidInput`] if:
    /// - Guest does not send packets in their expected order
    /// - Guest sets their intent to [`ConnectionIntent::Unknown`]
    /// 
    /// Returns [`io::ErrorKind::PermissionDenied`] if:
    /// - Guest fails the challenge
    /// - Guest sends invalid nonce
    /// 
    /// Delegates errors from [`Protocol::read_frame`] and [`Protocol::send_message`]
    pub async fn begin(&mut self) -> io::Result<(ConnectionIntent, String)> {
        if let HandshakePacketGuestToHost::Hello { connection_type } = self.state.protocol.read_frame().await? {
            self.connection_type = connection_type;

            self.state.protocol.send_message(HandshakePacketHostToGuest::Acknowledge {
                ok: true,
                err: None
            }).await?;

            if let HandshakePacketGuestToHost::Identify { hostname } = self.state.protocol.read_frame().await? {
                // todo: check whitelist/blacklist
                info!("Looking up challenge record for {hostname}");
                let resolver = TokioAsyncResolver::tokio(
                    ResolverConfig::default(),
                    ResolverOpts::default());
                let txt_resp = resolver.txt_lookup(format!("_osp.{hostname}")).await;
                match txt_resp {
                    Ok(txt_resp) => {
                        if let Some(record) = txt_resp.iter().next() {
                            info!("Challenge record found");
                            debug!("Challenge record: {record}");
                            let pub_key = Rsa::public_key_from_pem(record.to_string().as_bytes())?;

                            info!("Generating and encrypting challenge bytes");
                            let mut challenge_bytes = [0; 256];
                            rand_bytes(&mut challenge_bytes)?;
                            let mut encrypted_challenge = vec![0u8; pub_key.size() as usize];
                            pub_key.public_encrypt(&challenge_bytes, &mut encrypted_challenge, Padding::PKCS1)?;

                            info!("Sending challenge bytes");
                            self.state.protocol.send_message(HandshakePacketHostToGuest::Challenge {
                                encrypted_challenge,
                                nonce: self.state.nonce,
                            }).await?;

                            if let HandshakePacketGuestToHost::Verify { challenge, nonce } = self.state.protocol.read_frame().await? {
                                info!("Received challenge verification");
                                if nonce != self.state.nonce {
                                    error!("Challenge response had invalid nonce. Expected: {} Actual: {}. Rejecting...", self.state.nonce, nonce);
                                    return Err(self.send_close_err(io::ErrorKind::PermissionDenied, "Invalid nonce".to_string()).await);
                                }

                                if challenge == challenge_bytes {
                                    info!("Challenge verification successful");
                                    self.state.protocol.send_message(HandshakePacketHostToGuest::ChallengeResponse {
                                        successful: true,
                                    }).await?;
                                    debug!("Sent success packet.");
                                    if let HandshakePacketGuestToHost::SetIntent { intent } = self.state.protocol.read_frame().await? {
                                        if intent == ConnectionIntent::Unknown {
                                            warn!("Guest set unknown intent. Closing...");
                                            return Err(self.send_close_err(io::ErrorKind::InvalidInput, "Unknown intent".to_string()).await);
                                        }

                                        let can_continue = match intent {
                                            ConnectionIntent::Subscribe | ConnectionIntent::Unknown => false,
                                            ConnectionIntent::TransferData => true,
                                        };

                                        info!("Guest set intent {intent}, can continue: {can_continue}.");
                                        self.state.protocol.send_message(HandshakePacketHostToGuest::Close {
                                            can_continue,
                                            err: None,
                                        }).await?;

                                        Ok((intent, hostname))
                                    } else {
                                        warn!("Guest did not set intent. Closing...");
                                        Err(self.send_close_err(io::ErrorKind::InvalidInput, "Intent must be set".to_string()).await)
                                    }
                                } else {
                                    warn!("Guest failed challenge, bytes did not match. Closing...");
                                    Err(self.send_close_err(io::ErrorKind::PermissionDenied, "Challenge failed".to_string()).await)
                                }
                            } else {
                                warn!("Guest did not send challenge verification. Closing...");
                                Err(self.send_close_err(io::ErrorKind::InvalidInput, "Expected challenge verification packet".to_string()).await)
                            }
                        } else {
                            warn!("Failed to resolve SRV record for {hostname}. Closing...");
                            Err(
                                self.send_close_err(
                                    io::ErrorKind::InvalidData,
                                    format!("Failed to resolve TXT record for {hostname}. Is it located at _osp.{hostname}?")
                                ).await
                            )
                        }
                    }
                    Err(e) => {
                        warn!("Failed to resolve TXT record for {hostname}. Closing...\n\nFurther Details: {}", e.to_string());
                        Err(
                            self.send_close_err(
                                io::ErrorKind::InvalidData,
                                format!(
                                    "Failed to resolve TXT record for {hostname}. Is it located at _osp.{hostname}?"
                                )
                            ).await
                        )
                    }
                }
            } else {
                warn!("Guest did not identify. Closing...");
                Err(self.send_close_err(io::ErrorKind::InvalidInput, "Expected identify packet".to_string()).await)
            }
        } else {
            warn!("No hello packet received from guest. Closing...");
            Err(self.send_close_err(io::ErrorKind::InvalidInput, "Expected hello packet".to_string()).await)
        }
    }

    /// Switch connection into data transfer state 
    #[must_use] pub fn start_transfer(self) -> InboundConnection<TransferState> {
        InboundConnection {
            connection_type: self.connection_type.clone(),
            data_types: self.data_types.clone(),
            state: TransferState {
                protocol: self.state.protocol.map_codecs(
                    |_| {
                        PacketDecoder::new() // Transfer packet types implied!
                    },
                    |_| {
                        PacketEncoder::new()
                    }
                ),
            },
        }
    }
}

impl InboundConnection<TransferState> {
    pub fn start_recv() {
        todo!()
    }
}
