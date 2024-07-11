use std::ops::Deref;
use bytes::{Buf, BufMut, BytesMut};

use tokio::io;

use uuid::Uuid;

use crate::packet::{DeserializePacket, SerializePacket};

pub enum TransferPacketHostToGuest {
    AcknowledgeObject {
        can_send: bool
    }
}

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
            TransferPacketHostToGuest::AcknowledgeObject { can_send } => {
                buf.put_u8(*can_send as u8);
                bytes_written += 1;
            }
        }

        Ok(bytes_written)
    }
}

impl DeserializePacket for TransferPacketHostToGuest {
    type Output = TransferPacketHostToGuest;

    fn deserialize(buf: &mut BytesMut) -> io::Result<Self::Output> {
        match buf.get_u8() {
            1 => Ok(TransferPacketHostToGuest::AcknowledgeObject {
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
            TransferPacketGuestToHost::IdentifyObject { data_chunks, data_len, data_id } => {
                bytes_written += self.write_uuid(buf, data_id);
                buf.put_u64(*data_len as u64);
                buf.put_u64(*data_chunks as u64);
                bytes_written += 16; // two u64
            }
            TransferPacketGuestToHost::SendChunk { data, done } => {
                buf.put_u64(data.len() as u64);
                bytes_written += 8;
                buf.put_slice(data);
                bytes_written += data.len();
                buf.put_u8(*done as u8);
                bytes_written += 1;
            }
        }

        Ok(bytes_written)
    }
}

impl DeserializePacket for TransferPacketGuestToHost {
    type Output = TransferPacketGuestToHost;

    fn deserialize(buf: &mut BytesMut) -> io::Result<Self::Output> {
        match buf.get_u8() {
            1 => Ok(TransferPacketGuestToHost::IdentifyObject {
                data_id: Self::read_uuid(buf),
                data_len: buf.get_u64() as usize,
                data_chunks: buf.get_u64() as usize,
            }),
            2 => {
                let data_len = buf.get_u64() as usize;
                let mut data_buf = vec![0u8; data_len];
                buf.copy_to_slice(&mut data_buf);

                Ok(TransferPacketGuestToHost::SendChunk {
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
