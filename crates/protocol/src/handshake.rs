use std::pin::Pin;
use tokio::io::{self, AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
// use tokio_byteorder::{AsyncReadBytesExt,AsyncWriteBytesExt,NetworkEndian};
use uuid::Uuid;

use super::utils::{DeserializePacket, SerializePacket};

pub enum ConnectionType {
    Unknown = 0,
    Client = 1,
    Server = 2
}

impl ConnectionType {
    fn from_u8(t: u8) -> ConnectionType {
        match t {
            1 => ConnectionType::Client,
            2 => ConnectionType::Server,
            _ => ConnectionType::Unknown
        }
    }
}

impl From<&ConnectionType> for u8 {
    fn from(t: &ConnectionType) -> Self {
        match t {
            ConnectionType::Unknown => 0,
            ConnectionType::Client => 1,
            ConnectionType::Server => 2,
        }
    }
}

pub enum OSPHandshakeIn {
    Hello {
        connection_type: ConnectionType,
    },
    Identify {
        hostname: String,
    },
    Verify {
        challenge: Vec<u8>,
        nonce: Uuid,
    }
}

impl From<&OSPHandshakeIn> for u8 {
    fn from(req: &OSPHandshakeIn) -> Self {
        match req {
            OSPHandshakeIn::Hello { .. } => 1,
            OSPHandshakeIn::Identify { .. } => 2,
            OSPHandshakeIn::Verify { .. } => 3,
        }
    }
}

impl SerializePacket for OSPHandshakeIn {
    /// Serialize Request to bytes (to send to server)
    async fn serialize(&self, mut buf: Pin<&mut impl AsyncWrite>) -> io::Result<usize> {
        buf.write_u8(self.into()).await?; // Message Type byte
        let mut bytes_written: usize = 1;
        match self {
            OSPHandshakeIn::Hello { connection_type } => {
                buf.write_u8(u8::from(connection_type)).await?;
                bytes_written += 1
            }
            OSPHandshakeIn::Identify { hostname } => {
                bytes_written += self.write_string(buf, hostname).await?;
            }
            OSPHandshakeIn::Verify { challenge, nonce } => {
                buf.write_all(challenge).await?;
                bytes_written += 256;

                bytes_written += self.write_uuid(buf, nonce).await?;
            }
        }
        Ok(bytes_written)
    }
}

impl DeserializePacket for OSPHandshakeIn {
    type Output = OSPHandshakeIn;

    async fn deserialize(mut buf: Pin<&mut impl AsyncRead>) -> io::Result<OSPHandshakeIn> {
        // We'll match the same `u8` that is used to recognize which request type this is
        match buf.read_u8().await? {
            1 => Ok(OSPHandshakeIn::Hello {
                connection_type: ConnectionType::from_u8(buf.read_u8().await?),
            }),
            2 => Ok(OSPHandshakeIn::Identify {
                hostname: Self::read_string(buf).await?,
            }),
            3 => {
                let mut challenge_bytes = vec![0u8; 256];
                buf.read_exact(&mut challenge_bytes).await?;
                Ok(OSPHandshakeIn::Verify {
                    challenge: challenge_bytes,
                    nonce: Self::read_uuid(buf).await?,
                })
            },
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Request Type",
            )),
        }
    }
}

pub enum OSPHandshakeOut {
    Acknowledge {
        ok: bool,
        err: Option<String>,
    },
    Challenge {
        encrypted_challenge: Vec<u8>,
        nonce: Uuid,
    },
    Close {
        can_continue: bool,
        err: Option<String>
    }
}

impl From<&OSPHandshakeOut> for u8 {
    fn from(req: &OSPHandshakeOut) -> Self {
        match req {
            OSPHandshakeOut::Acknowledge { .. } => 1,
            OSPHandshakeOut::Challenge { .. } => 2,
            OSPHandshakeOut::Close { .. } => 3,
        }
    }
}

impl SerializePacket for OSPHandshakeOut {
    /// Serialize Response to bytes (to send to client)
    async fn serialize(&self, mut buf: Pin<&mut impl AsyncWrite>) -> io::Result<usize> {
        buf.write_u8(self.into()).await?; // Message Type byte
        let mut bytes_written: usize = 1;
        match self {
            OSPHandshakeOut::Acknowledge { ok, err } => {
                buf.write_u8(*ok as u8).await?;
                bytes_written += 1;

                bytes_written += self.write_optional_string(buf, err).await?;
            }
            OSPHandshakeOut::Challenge { encrypted_challenge: challenge_encrypted, nonce } => {
                buf.write_u16(challenge_encrypted.len() as u16).await?;
                bytes_written += 2;
                buf.write_all(challenge_encrypted).await?;

                bytes_written += self.write_uuid(buf, nonce).await?;
            }
            OSPHandshakeOut::Close { can_continue: ok, err} => {
                buf.write_u8(*ok as u8).await?;
                bytes_written += 1;

                bytes_written += self.write_optional_string(buf, err).await?;
            }
        }
        Ok(bytes_written)
    }
}

impl DeserializePacket for OSPHandshakeOut {
    type Output = OSPHandshakeOut;

    async fn deserialize(mut buf: Pin<&mut impl AsyncRead>) -> io::Result<OSPHandshakeOut> {
        // We'll match the same `u8` that is used to recognize which response type this is
        match buf.read_u8().await? {
            1 => Ok(OSPHandshakeOut::Acknowledge {
                ok: buf.read_u8().await? != 0,
                err: Self::read_optional_string(buf).await?,
            }),
            2 => {
                let challenge_len = buf.read_u16().await?;
                let mut challenge_encrypted = vec![0u8; challenge_len as usize];
                buf.read_exact(&mut challenge_encrypted).await?;

                Ok(OSPHandshakeOut::Challenge {
                    encrypted_challenge: challenge_encrypted,
                    nonce: Self::read_uuid(buf).await?,
                })
            },
            3 => Ok(OSPHandshakeOut::Close {
                can_continue: buf.read_u8().await? != 0,
                err: Self::read_optional_string(buf).await?,
            }),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Request Type",
            )),
        }
    }
}
