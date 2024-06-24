use std::marker::PhantomData;
use bytes::{Buf, BufMut, BytesMut};
use tokio::io;
use tokio_util::codec::{Decoder, Encoder};
use uuid::Uuid;

pub mod server;

const PACKET_MAX_LENGTH: usize = 8 * 1024 * 1024;

pub trait SerializePacket {
    /// Serialize to a `Write`able buffer
    fn serialize(&self, buf: &mut BytesMut) -> io::Result<usize>;

    fn write_string(&self, buf: &mut BytesMut, string: &String) -> usize where Self : Sized {
        let bytes = string.as_bytes();
        buf.put_u16(bytes.len() as u16);
        buf.put_slice(&bytes);
        2 + bytes.len() // u16 = 2 bytes
    }

    fn write_optional_string(&self, buf: &mut BytesMut, string: &Option<String>) -> usize where Self: Sized {
        buf.put_u8(string.is_some() as u8);
        let mut bytes_written = 1;
        if let Some(str) = string {
            bytes_written += self.write_string(buf, str);
        }
        bytes_written
    }

    fn write_uuid(&self, buf: &mut BytesMut, uuid: &Uuid) -> usize where Self: Sized {
        buf.put_u128(uuid.as_u128());
        16 // u128 is 16 bytes
    }

    fn write_optional_uuid(&self, buf: &mut BytesMut, uuid: &Option<Uuid>) -> usize where Self: Sized {
        buf.put_u8(uuid.is_some() as u8);
        let mut bytes_written = 1;
        if let Some(id) = uuid {
            bytes_written += self.write_uuid(buf, id);
        }
        bytes_written
    }
}

/// Trait for something that can be converted from bytes (&[u8])
pub trait DeserializePacket {
    /// The type that this deserializes to
    type Output;

    /// Deserialize from a `Read`able buffer
    fn deserialize(buf: &mut BytesMut) -> io::Result<Self::Output>;

    /// From a given readable buffer (TcpStream), read the next length (u16) and extract the string bytes ([u8])
    fn read_string(buf: &mut BytesMut) -> io::Result<String> {
        let length = buf.get_u16();

        // Given the length of our string, only read in that quantity of bytes
        let mut bytes = vec![0u8; length as usize];
        buf.copy_to_slice(&mut bytes);

        // And attempt to decode it as UTF8
        String::from_utf8(bytes).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid utf8"))
    }

    fn read_optional_string(buf: &mut BytesMut) -> io::Result<Option<String>> {
        Ok(if buf.get_u8() != 0 { // if the boolean is set read the optional value
            Some(Self::read_string(buf)?)
        } else { None })
    }

    fn read_uuid(buf: &mut BytesMut) -> Uuid {
        Uuid::from_u128(buf.get_u128())
    }

    fn read_optional_uuid(buf: &mut BytesMut) -> Option<Uuid> {
        if buf.get_u8() != 0 { // if the boolean is set read the optional value
            Some(Self::read_uuid(buf))
        } else { None }
    }
}

pub struct PacketCodec<PacketType: DeserializePacket + SerializePacket> {
    _packet_type: PhantomData<PacketType>
}

impl<PacketType: DeserializePacket + SerializePacket> PacketCodec<PacketType> {
    pub fn new() -> PacketCodec<PacketType> {
        PacketCodec::<PacketType> {
            _packet_type: PhantomData::default(),
        }
    }
}

impl<PacketType: DeserializePacket + SerializePacket> Encoder<PacketType> for PacketCodec<PacketType> {
    type Error = io::Error;

    fn encode(&mut self, item: PacketType, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut buf = &mut BytesMut::with_capacity(PACKET_MAX_LENGTH);
        item.serialize(& mut buf)?;

        if buf.len() > PACKET_MAX_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", buf.len())
            ));
        }

        // Convert the length into a byte array.
        // The cast to u32 cannot overflow due to the length check above.
        let len_slice = u32::to_le_bytes(buf.len() as u32);

        // Reserve space in the buffer.
        dst.reserve(4 + buf.len());

        // Write the length and string to the buffer.
        dst.extend_from_slice(&len_slice);
        dst.extend_from_slice(buf);
        Ok(())
    }
}


impl<PacketType: DeserializePacket + SerializePacket> Decoder for PacketCodec<PacketType> {
    type Item = PacketType::Output;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            // Not enough data to read length marker.
            return Ok(None);
        }

        // Read length marker.
        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_le_bytes(length_bytes) as usize;

        // Check that the length is not too large to avoid a denial of
        // service attack where the server runs out of memory.
        if length > PACKET_MAX_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", length)
            ));
        }

        if src.len() < 4 + length {
            // The full string has not yet arrived.
            //
            // We reserve more space in the buffer. This is not strictly
            // necessary, but is a good idea performance-wise.
            src.reserve(4 + length - src.len());

            // We inform the Framed that we need more bytes to form the next
            // frame.
            return Ok(None);
        }

        // Use advance to modify src such that it no longer contains
        // this frame.
        let data = src[4..4 + length].to_vec();
        src.advance(4 + length);

        let packet = PacketType::deserialize(&mut BytesMut::from(data.as_slice()))?;

        Ok(Some(packet))
    }
}