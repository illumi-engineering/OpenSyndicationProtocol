use tokio::io;
use tokio::sync::{Mutex};

use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc};
use bincode::Encode;
use bytes::BytesMut;

use log::{error, info, warn};

use openssl::pkey::Private;
use openssl::rsa::{Padding, Rsa};

use trust_dns_resolver::{TokioAsyncResolver};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use osp_data::Data;
use osp_data::registry::DataTypeRegistry;

use osp_protocol::{ConnectionType, OSPUrl, Protocol};
use osp_protocol::packet::handshake::{HandshakePacketGuestToHost, HandshakePacketHostToGuest};
use osp_protocol::packet::{PACKET_MAX_LENGTH, PacketDecoder, PacketEncoder};
use osp_protocol::packet::transfer::{TransferPacketHostToGuest, TransferPacketGuestToHost};
use osp_protocol::utils::ConnectionIntent;

/// Outbound connection
#[allow(clippy::module_name_repetitions)]
pub struct OutboundConnection<TState> {
    private_key: Rsa<Private>,
    hostname: String,
    data_types: Arc<Mutex<DataTypeRegistry>>,
    addr: SocketAddr,
    state: TState
}

pub struct WaitingState {}

pub struct HandshakeState {
    protocol: Protocol<HandshakePacketHostToGuest, HandshakePacketGuestToHost>, // packet types reversed
}

pub struct TransferState {
    protocol: Protocol<TransferPacketHostToGuest, TransferPacketGuestToHost>
}

impl OutboundConnection<WaitingState> {
    pub async fn create(url: OSPUrl, private_key: Rsa<Private>, hostname: String, data_types: Arc<Mutex<DataTypeRegistry>>) -> io::Result<Self> {
        info!("Resolving osp connection to {url}");
        let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

        let ip_resp = resolver.ipv4_lookup(url.domain.clone()).await?;
        if let Some(ip) = ip_resp.iter().next() {
            info!("Lookup successful, opening connection");
            Self::create_with_socket_addr(
                SocketAddr::new(IpAddr::from(ip.0), url.port),
                private_key,
                hostname,
                data_types,
            )
        } else {
            error!("Lookup failed");
            Err(io::Error::new(io::ErrorKind::NotConnected, format!("Failed to resolve address {}", url.domain)))
        }
    }

    pub fn create_with_socket_addr(addr: SocketAddr, private_key: Rsa<Private>, hostname: String, data_types: Arc<Mutex<DataTypeRegistry>>) -> io::Result<Self> {
        info!("Opening connection to {addr}");

        Ok(Self {
            private_key,
            hostname,
            data_types,
            addr,
            state: WaitingState {}
        })
    }

    pub async fn begin(&self) -> io::Result<OutboundConnection<HandshakeState>> {
        info!("Starting outbound connection");
        Ok(OutboundConnection {
            private_key: self.private_key.clone(),
            hostname: self.hostname.clone(),
            data_types: self.data_types.clone(),
            addr: self.addr,
            state: HandshakeState {
                protocol: Protocol::connect(self.addr).await?,
            },
        })
    }
}

impl OutboundConnection<HandshakeState> {
    async fn read_frame_and_handle_err(&mut self) -> io::Result<Option<HandshakePacketHostToGuest>> {
        let packet = self.state.protocol.read_frame().await?;
        match packet {
            HandshakePacketHostToGuest::Close { can_continue: false, err: Some(msg) } => {
                error!("Connection cannot continue.");
                error!("Error message received: {msg}");
                Ok(None)
            },
            packet => Ok(Some(packet))
        }
    }

    pub async fn handshake(&mut self) -> io::Result<()> {
        let addr = self.addr;
        info!("<{addr}> Starting outbound handshake");
        let hostname = self.hostname.clone();
        let private_key = self.private_key.clone();
        self.state.protocol.send_message(HandshakePacketGuestToHost::Hello { connection_type: ConnectionType::Server }).await?;

        if let Some(HandshakePacketHostToGuest::Acknowledge {
            ok,
            err
        }) = self.read_frame_and_handle_err().await? {
            if ok {
                info!("Handshake acknowledged");
                self.state.protocol.send_message(HandshakePacketGuestToHost::Identify {
                    hostname,
                }).await?;

                if let Some(HandshakePacketHostToGuest::Challenge {
                    nonce,
                    encrypted_challenge
                }) = self.read_frame_and_handle_err().await? {
                    info!("Challenge received, decrypting");
                    info!("Connection Nonce: {nonce}");
                    let mut decrypt_buf = vec![0u8; private_key.size() as usize];
                    private_key.private_decrypt(&encrypted_challenge, &mut decrypt_buf, Padding::PKCS1)?;

                    info!("Sending decrypted challenge");
                    self.state.protocol.send_message(HandshakePacketGuestToHost::Verify {
                        nonce,
                        challenge: decrypt_buf,
                    }).await?;

                    if matches!(self.read_frame_and_handle_err().await?, Some(HandshakePacketHostToGuest::ChallengeResponse {
                        successful: true,
                    })) {
                        info!("Handshake successful!");

                        Ok(())
                    } else {
                        Err(io::Error::new(io::ErrorKind::Other, "Unknown error occurred receiving challenge response from host"))
                    }
                } else {
                    Err(io::Error::new(io::ErrorKind::Other, "Unknown error occurred receiving challenge from host"))
                }
            } else {
                error!("Hello failed due to peer err: {}", err.clone().unwrap_or_else(|| "Unknown peer error".to_string()));
                Err(io::Error::new(io::ErrorKind::Other, err.unwrap_or_else(|| "Unknown peer error".to_string())))
            }
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Unknown error occurred receiving ACK from host"))
        }
    }

    pub async fn subscribe(&mut self) -> io::Result<()> {
        self.state.protocol.send_message(HandshakePacketGuestToHost::SetIntent {
            intent: ConnectionIntent::Subscribe
        }).await?;

        if matches!(self.read_frame_and_handle_err().await?, Some(HandshakePacketHostToGuest::Close {
            can_continue: false,
            err: None
        })) {
            Ok(())
        } else {
            warn!("Subscribe failed.");
            Err(io::Error::new(io::ErrorKind::Other, "Unknown error subscribing to host"))
        }
    }

    pub async fn start_transfer(mut self) -> io::Result<OutboundConnection<TransferState>> {
        self.state.protocol.send_message(HandshakePacketGuestToHost::SetIntent {
            intent: ConnectionIntent::TransferData
        }).await?;

        if matches!(self.read_frame_and_handle_err().await?, Some(HandshakePacketHostToGuest::Close {
            can_continue: true,
            err: None
        })) {
            Ok(OutboundConnection {
                data_types: self.data_types.clone(),
                addr: self.addr,
                hostname: self.hostname.clone(),
                private_key: self.private_key.clone(),
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
            })
        } else {
            warn!("Failed to enter transfer state.");
            Err(io::Error::new(io::ErrorKind::Other, "Unknown error switching to transfer state"))
        }
    }
}

impl OutboundConnection<TransferState> {
    /// Send a data object to the host.
    ///
    /// 
    pub async fn send_data<TData>(&mut self, obj: TData) -> io::Result<()>
    where
        TData : Data + Encode + 'static
    {
        let data_types = self.data_types.lock().await;
        let marshaller = data_types.by_type_id::<TData>();
        match marshaller {
            None => Err(io::Error::new(io::ErrorKind::Unsupported, format!("No marshaller registered for type with id {}", TData::get_id_static()))),
            Some(marshaller) => {
                let m = marshaller;

                let mut buf = BytesMut::new();
                let data_len = m.encode_to_bytes(&mut buf, obj)
                    .map_err(|e| { io::Error::new(io::ErrorKind::Other, e.to_string()) })?;
                let data_bytes = buf.to_vec();
                let data_id = TData::get_id_static();

                // the SendChunk packet needs two bytes for its own data (type, done) so the max
                // chunk length we can achieve can be determined by PACKET_MAX_LENGTH - 2
                let data_chunks = data_bytes.chunks(PACKET_MAX_LENGTH - 2);
                let chunks_len = data_chunks.len();

                self.state.protocol.send_message(TransferPacketGuestToHost::IdentifyObject {
                    data_id,
                    data_len,
                    data_chunks: chunks_len,
                }).await?;

                #[allow(irrefutable_let_patterns)]
                if let TransferPacketHostToGuest::AcknowledgeObject { can_send } = self.state.protocol.read_frame().await? {
                    if can_send {
                        for (i, chunk) in data_chunks.enumerate() {
                            let chunk_vec = Vec::from(chunk);

                            self.state.protocol.send_message(TransferPacketGuestToHost::SendChunk {
                                data: chunk_vec,
                                done: i == chunks_len - 1,
                            }).await?;
                        }
                    } else {
                        warn!("Failed to send data to inbound client, client is not accepting data type {data_id}.");
                    }
                }

                Ok(())
            }
        }
    }
}