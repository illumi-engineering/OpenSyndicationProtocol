use tokio::io;

use std::net::{IpAddr, SocketAddr};

use log::{error, info};

use openssl::pkey::Private;
use openssl::rsa::{Padding, Rsa};

use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};

use url::quirks::port;

use osp_protocol::{ConnectionType, OSPUrl, Protocol};
use osp_protocol::packet::{DeserializePacket, SerializePacket};
use osp_protocol::packet::handshake::{HandshakePacketGuestToHost, HandshakePacketHostToGuest};

pub struct OutboundConnection<TState> {
    private_key: Rsa<Private>,
    hostname: String,
    addr: SocketAddr,
    state: TState
}

pub struct WaitingState {}

pub struct HandshakeState {
    protocol: Protocol<HandshakePacketHostToGuest, HandshakePacketGuestToHost>, // packet types reversed
}

impl OutboundConnection<WaitingState> {
    pub fn create(url: OSPUrl, private_key: Rsa<Private>, hostname: String) -> io::Result<Self> {
        info!("Resolving osp connection to {url}");
        let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();

        let ip_resp = resolver.ipv4_lookup(url.domain.clone())?;
        if let Some(ip) = ip_resp.iter().next() {
            info!("Lookup successful, opening connection");
            Self::create_with_socket_addr(
                SocketAddr::new(IpAddr::from(ip.0), url.port),
                private_key,
                hostname
            )
        } else {
            error!("Lookup failed");
            Err(io::Error::new(io::ErrorKind::NotConnected, format!("Failed to resolve address {}", url.domain)))
        }
    }

    pub fn create_with_socket_addr(addr: SocketAddr, private_key: Rsa<Private>, hostname: String) -> io::Result<Self> {
        info!("Opening connection to {addr}");

        Ok(Self {
            private_key,
            hostname,
            addr,
            state: WaitingState {}
        })
    }

    pub async fn begin(&mut self) -> io::Result<OutboundConnection<HandshakeState>> {
        info!("Starting outbound connection");
        Ok(OutboundConnection {
            private_key: self.private_key.clone(),
            hostname: self.hostname.clone(),
            addr: self.addr.clone(),
            state: HandshakeState {
                protocol: Protocol::connect(self.addr).await?,
            },
        })
    }
}

impl OutboundConnection<HandshakeState> {
    pub async fn handshake(&mut self) -> io::Result<()> {
        let addr = self.addr;
        info!("<{addr}> Starting outbound handshake");
        let hostname = self.hostname.clone();
        let private_key = self.private_key.clone();
        self.state.protocol.send_message(HandshakePacketGuestToHost::Hello { connection_type: ConnectionType::Server }).await?;

        if let HandshakePacketHostToGuest::Acknowledge {
            ok,
            err
        } = self.state.protocol.read_frame().await? {
            if ok {
                info!("Handshake acknowledged");
                self.state.protocol.send_message(HandshakePacketGuestToHost::Identify {
                    hostname,
                }).await?;

                if let HandshakePacketHostToGuest::Challenge {
                    nonce,
                    encrypted_challenge
                } = self.state.protocol.read_frame().await? {
                    info!("Challenge received, decrypting");
                    let mut decrypt_buf = vec![0u8; private_key.size() as usize];
                    private_key.private_decrypt(&*encrypted_challenge, &mut *decrypt_buf, Padding::PKCS1)?;

                    info!("Sending decrypted challenge");
                    self.state.protocol.send_message(HandshakePacketGuestToHost::Verify {
                        nonce,
                        challenge: decrypt_buf,
                    }).await?;

                    if let HandshakePacketHostToGuest::Close {
                        can_continue,
                        err,
                    } = self.state.protocol.read_frame().await? {
                        if can_continue {
                            info!("Handshake successful!")
                        } else {
                            match err {
                                None => {
                                    error!("Unknown handshake err");
                                }
                                Some(e) => {
                                    error!("Handshake err: {}", e);
                                }
                            }
                        }
                    }
                }
            } else {
                error!("Hello failed: {}", err.unwrap());
            }
        }
        Ok(())
    }

}