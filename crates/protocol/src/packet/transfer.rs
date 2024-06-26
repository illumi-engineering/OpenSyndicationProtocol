use bytes::BytesMut;
use crate::{DeserializePacket, SerializePacket};
pub enum TransferPacket {

}

impl SerializePacket for TransferPacket {
    fn serialize(&self, buf: &mut BytesMut) -> std::io::Result<usize> {
        todo!()
    }
}

impl DeserializePacket for TransferPacket {
    type Output =TransferPacket;

    fn deserialize(buf: &mut BytesMut) -> std::io::Result<Self::Output> {
        todo!()
    }
}