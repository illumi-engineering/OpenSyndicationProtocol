use bytes::{Buf, BufMut, BytesMut};
use crate::packet::{DeserializePacket, SerializePacket};

pub struct DataPacket {
    length: usize,
    data: Vec<u8>,
}

impl SerializePacket for DataPacket {
    fn serialize(&self, buf: &mut BytesMut) -> std::io::Result<usize> {
        let mut bytes_written = 0;

        buf.put_u64(self.length as u64);
        bytes_written += 8;

        buf.put_slice(self.data.as_slice());
        bytes_written += self.data.len();

        Ok(bytes_written)
    }
}

impl DeserializePacket for DataPacket {
    type Output = Self;

    fn deserialize(buf: &mut BytesMut) -> std::io::Result<Self::Output> {
        let length = buf.get_u64() as usize;

        let mut data = vec![0u8; length];
        buf.copy_to_slice(&mut data);

        Ok(DataPacket {
            length,
            data,
        })
    }
}