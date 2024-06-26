use bytes::{Buf, BufMut, BytesMut};
use tokio::io;
// use tokio_byteorder::{AsyncReadBytesExt,AsyncWriteBytesExt,NetworkEndian};
use uuid::Uuid;
use crate::ConnectionType;
use crate::packet::{DeserializePacket, SerializePacket};

pub enum HandshakePacket {
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

impl From<&HandshakePacket> for u8 {
    fn from(pkt: &HandshakePacket) -> Self {
        match pkt {
            HandshakePacket::Hello { .. } => 1,
            HandshakePacket::Identify { .. } => 2,
            HandshakePacket::Verify { .. } => 3,
            HandshakePacket::Acknowledge { .. } => 4,
            HandshakePacket::Challenge { .. } => 5,
            HandshakePacket::Close { .. } => 6,
        }
    }
}

impl SerializePacket for HandshakePacket {
    /// Serialize Request to bytes (to send to server)
    fn serialize(&self, buf: &mut BytesMut) -> io::Result<usize> {
        buf.put_u8(self.into()); // Message Type byte
        let mut bytes_written: usize = 1;
        match self {
            HandshakePacket::Hello { connection_type } => {
                buf.put_u8(u8::from(connection_type));
                bytes_written += 1
            }
            HandshakePacket::Identify { hostname } => {
                bytes_written += self.write_string(buf, hostname);
            }
            HandshakePacket::Verify { challenge, nonce } => {
                // since this is always 256 bytes we can leave the len header out
                buf.put_slice(challenge);
                bytes_written += challenge.len();

                bytes_written += self.write_uuid(buf, nonce);
            }
            HandshakePacket::Acknowledge { ok, err } => {
                buf.put_u8(*ok as u8);
                bytes_written += 1;

                bytes_written += self.write_optional_string(buf, err);
            }
            HandshakePacket::Challenge { encrypted_challenge, nonce } => {
                buf.put_u16(encrypted_challenge.len() as u16);
                bytes_written += 2;
                buf.put_slice(encrypted_challenge);
                bytes_written += encrypted_challenge.len();

                bytes_written += self.write_uuid(buf, nonce);
            }
            HandshakePacket::Close { can_continue: ok, err} => {
                buf.put_u8(*ok as u8);
                bytes_written += 1;

                bytes_written += self.write_optional_string(buf, err);
            }
        }
        Ok(bytes_written)
    }
}

impl DeserializePacket for HandshakePacket {
    type Output = HandshakePacket;

    fn deserialize(buf: &mut BytesMut) -> io::Result<Self::Output> {
        // We'll match the same `u8` that is used to recognize which request type this is
        match buf.get_u8() {
            1 => Ok(HandshakePacket::Hello {
                connection_type: ConnectionType::from_u8(buf.get_u8()),
            }),
            2 => Ok(HandshakePacket::Identify {
                hostname: Self::read_string(buf)?,
            }),
            3 => {
                let mut challenge_bytes = vec![0u8; 256];
                buf.copy_to_slice(&mut challenge_bytes);
                Ok(HandshakePacket::Verify {
                    challenge: challenge_bytes,
                    nonce: Self::read_uuid(buf),
                })
            },
            4 => Ok(HandshakePacket::Acknowledge {
                ok: buf.get_u8() != 0,
                err: Self::read_optional_string(buf)?,
            }),
            5 => {
                let challenge_len = buf.get_u16();
                let mut challenge_encrypted = vec![0u8; challenge_len as usize];
                buf.copy_to_slice(&mut challenge_encrypted);

                Ok(HandshakePacket::Challenge {
                    encrypted_challenge: challenge_encrypted,
                    nonce: Self::read_uuid(buf),
                })
            },
            6 => Ok(HandshakePacket::Close {
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

