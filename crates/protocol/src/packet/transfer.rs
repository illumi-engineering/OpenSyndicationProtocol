use bytes::{Buf, BufMut, BytesMut};

use tokio::io;

use uuid::Uuid;

use crate::packet::{DeserializePacket, SerializePacket};

#[allow(clippy::module_name_repetitions)]
pub enum TransferPacketHostToGuest {
    AcknowledgeObject {
        can_send: bool
    }
}

#[allow(clippy::module_name_repetitions)]
pub enum TransferPacketGuestToHost {
    IdentifyObject {
        data_id: Uuid,
        data_len: usize,
        data_chunks: usize,
    },
    SendChunk {
        data: Vec<u8>,
        done: bool,
    }
}



impl From<&TransferPacketHostToGuest> for u8 {
    fn from(pkt: &TransferPacketHostToGuest) -> Self {
        match pkt {
            TransferPacketHostToGuest::AcknowledgeObject { .. } => 1
        }
    }
}

impl SerializePacket for TransferPacketHostToGuest {
    fn serialize(&self, buf: &mut BytesMut) -> io::Result<usize> {
        buf.put_u8(self.into());
        let mut bytes_written = 1;
        match self {
            Self::AcknowledgeObject { can_send } => {
                buf.put_u8(u8::from(*can_send));
                bytes_written += 1;
            }
        }

        Ok(bytes_written)
    }
}

impl DeserializePacket for TransferPacketHostToGuest {
    type Output = Self;

    fn deserialize(buf: &mut BytesMut) -> io::Result<Self::Output> {
        match buf.get_u8() {
            1 => Ok(Self::AcknowledgeObject {
                can_send: buf.get_u8() != 0,
            }),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Request Type",
            ))
        }
    }
}

impl From<&TransferPacketGuestToHost> for u8 {
    fn from(pkt: &TransferPacketGuestToHost) -> Self {
        match pkt {
            TransferPacketGuestToHost::IdentifyObject { .. } => 1,
            TransferPacketGuestToHost::SendChunk { .. } => 2
        }
    }
}

impl SerializePacket for TransferPacketGuestToHost {
    fn serialize(&self, buf: &mut BytesMut) -> io::Result<usize> {
        buf.put_u8(self.into());
        let mut bytes_written = 1;

        match self {
            Self::IdentifyObject { data_chunks, data_len, data_id } => {
                bytes_written += self.write_uuid(buf, data_id);
                buf.put_u64(*data_len as u64);
                buf.put_u64(*data_chunks as u64);
                bytes_written += 16; // two u64
            }
            Self::SendChunk { data, done } => {
                buf.put_u64(data.len() as u64);
                bytes_written += 8;
                buf.put_slice(data);
                bytes_written += data.len();
                buf.put_u8(u8::from(*done));
                bytes_written += 1;
            }
        }

        Ok(bytes_written)
    }
}

impl DeserializePacket for TransferPacketGuestToHost {
    type Output = Self;

    fn deserialize(buf: &mut BytesMut) -> io::Result<Self::Output> {
        match buf.get_u8() {
            1 => Ok(Self::IdentifyObject {
                data_id: Self::read_uuid(buf),
                #[allow(clippy::cast_possible_truncation)]
                data_len: buf.get_u64() as usize,
                #[allow(clippy::cast_possible_truncation)]
                data_chunks: buf.get_u64() as usize,
            }),
            2 => {
                #[allow(clippy::cast_possible_truncation)]
                let data_len = buf.get_u64() as usize;
                let mut data_buf = vec![0u8; data_len];
                buf.copy_to_slice(&mut data_buf);

                Ok(Self::SendChunk {
                    data: data_buf,
                    done: buf.get_u8() != 0,
                })
            },
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Request Type",
            ))
        }
    }
}
