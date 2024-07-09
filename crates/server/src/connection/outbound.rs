use tokio::io;

use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};

use log::{error, info};

use openssl::pkey::Private;
use openssl::rsa::{Padding, Rsa};

use trust_dns_resolver::{TokioAsyncResolver};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use osp_data::registry::DataTypeRegistry;

use osp_protocol::{ConnectionType, OSPUrl, Protocol};
use osp_protocol::packet::handshake::{HandshakePacketGuestToHost, HandshakePacketHostToGuest};

pub struct OutboundConnection<TState> {
    private_key: Rsa<Private>,
    hostname: String,
    data_marshallers: Arc<Mutex<DataTypeRegistry>>,
    addr: SocketAddr,
    state: TState
}

pub struct WaitingState {}

pub struct HandshakeState {
    protocol: Protocol<HandshakePacketHostToGuest, HandshakePacketGuestToHost>, // packet types reversed
}

impl OutboundConnection<WaitingState> {
    pub async fn create(url: OSPUrl, private_key: Rsa<Private>, hostname: String, data_marshallers: Arc<Mutex<DataTypeRegistry>>) -> io::Result<Self> {
        info!("Resolving osp connection to {url}");
        let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

        let ip_resp = resolver.ipv4_lookup(url.domain.clone()).await?;
        if let Some(ip) = ip_resp.iter().next() {
            info!("Lookup successful, opening connection");
            Self::create_with_socket_addr(
                SocketAddr::new(IpAddr::from(ip.0), url.port),
                private_key,
                hostname,
                data_marshallers,
            )
        } else {
            error!("Lookup failed");
            Err(io::Error::new(io::ErrorKind::NotConnected, format!("Failed to resolve address {}", url.domain)))
        }
    }

    pub fn create_with_socket_addr(addr: SocketAddr, private_key: Rsa<Private>, hostname: String, data_marshallers: Arc<Mutex<DataTypeRegistry>>) -> io::Result<Self> {
        info!("Opening connection to {addr}");

        Ok(Self {
            private_key,
            hostname,
            data_marshallers,
            addr,
            state: WaitingState {}
        })
    }

    pub async fn begin(&mut self) -> io::Result<OutboundConnection<HandshakeState>> {
        info!("Starting outbound connection");
        Ok(OutboundConnection {
            private_key: self.private_key.clone(),
            hostname: self.hostname.clone(),
            data_marshallers: self.data_marshallers.clone(),
            addr: self.addr.clone(),
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
            HandshakePacketHostToGuest::Close { can_continue: false, err } => {
                error!("Connection cannot continue.");
                if let Some(msg) = err {
                    error!("Error message received: {msg}");
                }
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
                    private_key.private_decrypt(&*encrypted_challenge, &mut *decrypt_buf, Padding::PKCS1)?;

                    info!("Sending decrypted challenge");
                    self.state.protocol.send_message(HandshakePacketGuestToHost::Verify {
                        nonce,
                        challenge: decrypt_buf,
                    }).await?;

                    if let Some(HandshakePacketHostToGuest::Close {
                        can_continue: true,
                        err: _,
                    }) = self.read_frame_and_handle_err().await? {
                        info!("Handshake successful!")
                    }
                }
            } else {
                error!("Hello failed: {}", err.unwrap());
            }
        }
        Ok(())
    }

}