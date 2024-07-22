use std::io::ErrorKind;
use bytes::{Buf, BufMut, BytesMut};
use tokio::io;
use crate::packet::{DeserializePacket, SerializePacket};


#[allow(clippy::module_name_repetitions)]
pub struct DataPacket {
    length: usize,
    data: Vec<u8>,
}

impl SerializePacket for DataPacket {
    fn serialize(&self, buf: &mut BytesMut) -> io::Result<usize> {
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

    fn deserialize(buf: &mut BytesMut) -> io::Result<Self::Output> {
        let length = usize::try_from(buf.get_u64())
            .map_err(|e| { io::Error::new(ErrorKind::InvalidData, e.to_string()) })?;

        let mut data = vec![0u8; length];
        buf.copy_to_slice(&mut data);

        Ok(Self {
            length,
            data,
        })
    }
}