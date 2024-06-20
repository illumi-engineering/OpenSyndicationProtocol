use std::io::{self, Read, Write};
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use uuid::Uuid;

pub trait SerializePacket {
    /// Serialize to a `Write`able buffer
    fn serialize(&self, buf: &mut impl Write) -> io::Result<usize>;

    fn write_string(&self, buf: &mut impl Write, string: &String) -> usize where Self : Sized {
        let bytes = string.as_bytes();
        buf.write_u16::<NetworkEndian>(bytes.len() as u16).unwrap();
        buf.write_all(&bytes).unwrap();
        2 + bytes.len() // u16 = 2 bytes
    }

    fn write_optional_string(&self, buf: &mut impl Write, string: &Option<String>) -> usize where Self: Sized {
        buf.write_u8(string.is_some() as u8).unwrap();
        let mut bytes_written = 1;
        if let Some(str) = string {
            bytes_written += self.write_string(buf, str);
        }
        bytes_written
    }

    fn write_uuid(&self, buf: &mut impl Write, uuid: &Uuid) -> usize where Self: Sized {
        buf.write_u128::<NetworkEndian>(uuid.as_u128()).unwrap();
        16 // u128 is 16 bytes
    }

    fn write_optional_uuid(&self, buf: &mut impl Write, uuid: &Option<Uuid>) -> usize where Self: Sized {
        buf.write_u8(uuid.is_some() as u8).unwrap();
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
    fn deserialize(buf: &mut impl Read) -> io::Result<Self::Output>;

    /// From a given readable buffer (TcpStream), read the next length (u16) and extract the string bytes ([u8])
    fn read_string(buf: &mut impl Read) -> io::Result<String> {
        let length = buf.read_u16::<NetworkEndian>().unwrap();

        // Given the length of our string, only read in that quantity of bytes
        let mut bytes = vec![0u8; length as usize];
        buf.read_exact(&mut bytes).unwrap();

        // And attempt to decode it as UTF8
        String::from_utf8(bytes).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid utf8"))
    }

    fn read_optional_string(buf: &mut impl Read) -> Option<String> {
        if buf.read_u8().unwrap() != 0 { // if the boolean is set read the optional value
            Some(Self::read_string(buf).unwrap())
        } else { None }
    }

    fn read_uuid(buf: &mut impl Read) -> Uuid {
        Uuid::from_u128(buf.read_u128::<NetworkEndian>().unwrap())
    }

    fn read_optional_uuid(buf: &mut impl Read) -> Option<Uuid> {
        if buf.read_u8().unwrap() != 0 { // if the boolean is set read the optional value
            Some(Self::read_uuid(buf))
        } else { None }
    }
}