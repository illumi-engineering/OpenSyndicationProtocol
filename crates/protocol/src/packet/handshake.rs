//! # Handshake Packets
//!

use bytes::{Buf, BufMut, BytesMut};

use tokio::io;

use uuid::Uuid;

use crate::ConnectionType;
use crate::packet::{DeserializePacket, SerializePacket};
use crate::utils::ConnectionIntent;

pub enum HandshakePacketGuestToHost {
    /// Establish a connection with the other server
    Hello {
        connection_type: ConnectionType,
    },

    /// Send my hostname to the other server
    Identify {
        hostname: String,
    },

    /// Send the client-decrypted challenge bytes back to the server
    Verify {
        challenge: Vec<u8>,
        nonce: Uuid,
    },

    /// Inform the other server of my intent.
    SetIntent {
        intent: ConnectionIntent,
    }
}

pub enum HandshakePacketHostToGuest {
    /// Acknowledge the client connection
    Acknowledge {
        ok: bool,
        err: Option<String>,
    },

    /// Send the challenge bytes to the client to decrypt
    Challenge {
        encrypted_challenge: Vec<u8>,
        nonce: Uuid,
    },

    /// Tell the client whether the challenge was successful
    ChallengeResponse {
        successful: bool,
    },

    /// Close the Handshake frame
    Close {
        can_continue: bool,
        err: Option<String>
    },
}

impl From<&HandshakePacketGuestToHost> for u8 {
    fn from(pkt: &HandshakePacketGuestToHost) -> Self {
        match pkt {
            HandshakePacketGuestToHost::Hello { .. } => 1,
            HandshakePacketGuestToHost::Identify { .. } => 2,
            HandshakePacketGuestToHost::Verify { .. } => 3,
            HandshakePacketGuestToHost::SetIntent { .. } => 4,
        }
    }
}

impl From<&HandshakePacketHostToGuest> for u8 {
    fn from(pkt: &HandshakePacketHostToGuest) -> Self {
        match pkt {
            HandshakePacketHostToGuest::Acknowledge { .. } => 1,
            HandshakePacketHostToGuest::Challenge { .. } => 2,
            HandshakePacketHostToGuest::Close { .. } => 3,
            HandshakePacketHostToGuest::ChallengeResponse { .. } => 4,
        }
    }
}

impl SerializePacket for HandshakePacketGuestToHost {
    /// Serialize Request to bytes (to send to server)
    fn serialize(&self, buf: &mut BytesMut) -> io::Result<usize> {
        buf.put_u8(self.into()); // Message Type byte
        let mut bytes_written: usize = 1;
        match self {
            HandshakePacketGuestToHost::Hello { connection_type } => {
                buf.put_u8(connection_type.into());
                bytes_written += 1
            }
            HandshakePacketGuestToHost::Identify { hostname } => {
                bytes_written += self.write_string(buf, hostname);
            }
            HandshakePacketGuestToHost::Verify { challenge, nonce } => {
                bytes_written += self.write_uuid(buf, nonce);

                // since this is always 256 bytes we can leave the len header out
                buf.put_slice(challenge);
                bytes_written += 256;
            }
            HandshakePacketGuestToHost::SetIntent { intent } => {
                buf.put_u8(intent.into());
                bytes_written += 1;
            }
        }
        Ok(bytes_written)
    }
}

impl SerializePacket for HandshakePacketHostToGuest {
    fn serialize(&self, buf: &mut BytesMut) -> io::Result<usize> {
        buf.put_u8(self.into()); // Message Type byte
        let mut bytes_written: usize = 1;
        match self {
            HandshakePacketHostToGuest::Acknowledge { ok, err } => {
                buf.put_u8(*ok as u8);
                bytes_written += 1;

                bytes_written += self.write_optional_string(buf, err);
            }
            HandshakePacketHostToGuest::Challenge { encrypted_challenge, nonce } => {
                buf.put_u16(encrypted_challenge.len() as u16);
                bytes_written += 2;
                buf.put_slice(encrypted_challenge);
                bytes_written += encrypted_challenge.len();

                bytes_written += self.write_uuid(buf, nonce);
            }
            HandshakePacketHostToGuest::Close { can_continue: ok, err} => {
                buf.put_u8(*ok as u8);
                bytes_written += 1;

                bytes_written += self.write_optional_string(buf, err);
            }
            HandshakePacketHostToGuest::ChallengeResponse { successful } => {
                buf.put_u8(*successful as u8);
                bytes_written += 1;
            }
        }

        Ok(bytes_written)
    }
}

impl DeserializePacket for HandshakePacketGuestToHost {
    type Output = HandshakePacketGuestToHost;

    fn deserialize(buf: &mut BytesMut) -> io::Result<Self::Output> {
        // We'll match the same `u8` that is used to recognize which request type this is
        match buf.get_u8() {
            1 => Ok(HandshakePacketGuestToHost::Hello {
                connection_type: ConnectionType::from(buf.get_u8()),
            }),
            2 => Ok(HandshakePacketGuestToHost::Identify {
                hostname: Self::read_string(buf)?,
            }),
            3 => {
                let nonce = Self::read_uuid(buf);
                let mut challenge_bytes = vec![0u8; 256];
                buf.copy_to_slice(&mut challenge_bytes);

                Ok(HandshakePacketGuestToHost::Verify {
                    challenge: challenge_bytes,
                    nonce,
                })
            },
            4 => Ok(HandshakePacketGuestToHost::SetIntent {
                intent: ConnectionIntent::from(buf.get_u8()),
            }),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Request Type",
            )),
        }
    }
}

impl DeserializePacket for HandshakePacketHostToGuest {
    type Output = HandshakePacketHostToGuest;

    fn deserialize(buf: &mut BytesMut) -> io::Result<Self::Output> {
        match buf.get_u8() {
            1 => Ok(HandshakePacketHostToGuest::Acknowledge {
                ok: buf.get_u8() != 0,
                err: Self::read_optional_string(buf)?,
            }),
            2 => {
                let challenge_len = buf.get_u16();
                let mut challenge_encrypted = vec![0u8; challenge_len as usize];
                buf.copy_to_slice(&mut challenge_encrypted);

                Ok(HandshakePacketHostToGuest::Challenge {
                    encrypted_challenge: challenge_encrypted,
                    nonce: Self::read_uuid(buf),
                })
            },
            3 => Ok(HandshakePacketHostToGuest::Close {
                can_continue: buf.get_u8() != 0,
                err: Self::read_optional_string(buf)?,
            }),
            4 => Ok(HandshakePacketHostToGuest::ChallengeResponse {
                successful: buf.get_u8() != 0,
            }),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Request Type",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    async fn serialize_gth_hello() {

    }
}
