use bytes::{Buf, BufMut, BytesMut};
use tokio::io;
// use tokio_byteorder::{AsyncReadBytesExt,AsyncWriteBytesExt,NetworkEndian};
use uuid::Uuid;
use crate::ConnectionType;
use crate::packet::{DeserializePacket, SerializePacket};

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

pub enum OSPServerProtocol {
    // in
    Hello {
        connection_type: ConnectionType,
    },
    Identify {
        hostname: String,
    },
    Verify {
        challenge: Vec<u8>,
        nonce: Uuid,
    },

    // out
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
    },
}

impl From<&OSPServerProtocol> for u8 {
    fn from(pkt: &OSPServerProtocol) -> Self {
        match pkt {
            OSPServerProtocol::Hello { .. } => 1,
            OSPServerProtocol::Identify { .. } => 2,
            OSPServerProtocol::Verify { .. } => 3,
            OSPServerProtocol::Acknowledge { .. } => 4,
            OSPServerProtocol::Challenge { .. } => 5,
            OSPServerProtocol::Close { .. } => 6,
        }
    }
}

impl SerializePacket for OSPServerProtocol {
    /// Serialize Request to bytes (to send to server)
    fn serialize(&self, buf: &mut BytesMut) -> io::Result<usize> {
        buf.put_u8(self.into()); // Message Type byte
        let mut bytes_written: usize = 1;
        match self {
            OSPServerProtocol::Hello { connection_type } => {
                buf.put_u8(u8::from(connection_type));
                bytes_written += 1
            }
            OSPServerProtocol::Identify { hostname } => {
                bytes_written += self.write_string(buf, hostname);
            }
            OSPServerProtocol::Verify { challenge, nonce } => {
                // buf.put_u16(256)?; // length of challenge bytes
                // bytes_written += 2;
                buf.put_slice(challenge);
                bytes_written += challenge.len();

                bytes_written += self.write_uuid(buf, nonce);
            }
            OSPServerProtocol::Acknowledge { ok, err } => {
                buf.put_u8(*ok as u8);
                bytes_written += 1;

                bytes_written += self.write_optional_string(buf, err);
            }
            OSPServerProtocol::Challenge { encrypted_challenge, nonce } => {
                buf.put_u16(encrypted_challenge.len() as u16);
                bytes_written += 2;
                buf.put_slice(encrypted_challenge);
                bytes_written += encrypted_challenge.len();

                bytes_written += self.write_uuid(buf, nonce);
            }
            OSPServerProtocol::Close { can_continue: ok, err} => {
                buf.put_u8(*ok as u8);
                bytes_written += 1;

                bytes_written += self.write_optional_string(buf, err);
            }
        }
        Ok(bytes_written)
    }
}

impl DeserializePacket for OSPServerProtocol {
    type Output = OSPServerProtocol;

    fn deserialize(buf: &mut BytesMut) -> io::Result<Self::Output> {
        // We'll match the same `u8` that is used to recognize which request type this is
        match buf.get_u8() {
            1 => Ok(OSPServerProtocol::Hello {
                connection_type: ConnectionType::from_u8(buf.get_u8()),
            }),
            2 => Ok(OSPServerProtocol::Identify {
                hostname: Self::read_string(buf)?,
            }),
            3 => {
                let mut challenge_bytes = vec![0u8; 256];
                buf.copy_to_slice(&mut challenge_bytes);
                Ok(OSPServerProtocol::Verify {
                    challenge: challenge_bytes,
                    nonce: Self::read_uuid(buf),
                })
            },
            4 => Ok(OSPServerProtocol::Acknowledge {
                ok: buf.get_u8() != 0,
                err: Self::read_optional_string(buf)?,
            }),
            5 => {
                let challenge_len = buf.get_u16();
                let mut challenge_encrypted = vec![0u8; challenge_len as usize];
                buf.copy_to_slice(&mut challenge_encrypted);

                Ok(OSPServerProtocol::Challenge {
                    encrypted_challenge: challenge_encrypted,
                    nonce: Self::read_uuid(buf),
                })
            },
            6 => Ok(OSPServerProtocol::Close {
                can_continue: buf.get_u8() != 0,
                err: Self::read_optional_string(buf)?,
            }),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Request Type",
            )),
        }
    }
}

