use std::io;
use std::net::{IpAddr, SocketAddr};
use log::{error, info};
use openssl::pkey::Private;
use openssl::rsa::{Padding, Rsa};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::Resolver;
use url::quirks::port;
use osp_protocol::{ConnectionType, OSPHandshakeIn, OSPHandshakeOut, OSPUrl, Protocol};

pub struct OutboundConnection {
    protocol: Protocol,
    private_key: Rsa<Private>,
    hostname: String,
}

impl OutboundConnection {
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
            protocol: Protocol::connect(addr)?,
            private_key,
            hostname,
        })
    }

    pub fn begin(&mut self) -> io::Result<()> {
        info!("Starting outbound handshake");
        let hostname = self.hostname.clone();
        let private_key = self.private_key.clone();
        self.protocol.send_message(&OSPHandshakeIn::Hello { connection_type: ConnectionType::Server })?;

        // if let OSPHandshakeOut::Acknowledge {
        //     ok,
        //     err
        // } = self.protocol.read_message::<OSPHandshakeOut>()? {
        //     if ok {
        //         info!("Handshake acknowledged");
        //         self.protocol.send_message(&OSPHandshakeIn::Identify {
        //             hostname,
        //         })?;
        //
        //         if let OSPHandshakeOut::Challenge {
        //             nonce,
        //             encrypted_challenge
        //         } = self.protocol.read_message::<OSPHandshakeOut>()? {
        //             info!("Challenge received, decrypting");
        //             let mut decrypt_buf = vec![0u8; private_key.size() as usize];
        //             private_key.private_decrypt(&*encrypted_challenge, &mut *decrypt_buf, Padding::PKCS1)?;
        //
        //             info!("Sending decrypted challenge");
        //             self.protocol.send_message(&OSPHandshakeIn::Verify {
        //                 nonce,
        //                 challenge: decrypt_buf,
        //             })?;
        //
        //             if let OSPHandshakeOut::Close {
        //                 can_continue,
        //                 err,
        //             } = self.protocol.read_message::<OSPHandshakeOut>()? {
        //                 if can_continue {
        //                     info!("Handshake successful!")
        //                 } else {
        //                     match err {
        //                         None => {
        //                             error!("Unknown handshake err");
        //                         }
        //                         Some(e) => {
        //                             error!("Handshake err: {}", e);
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //     } else {
        //         error!("Hello failed: {}", err.unwrap());
        //     }
        // }

        Ok(())
    }
}