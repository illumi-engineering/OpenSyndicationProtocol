use std::io::{self, Read, Write};
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};

pub trait SerializePacket {
    /// Serialize to a `Write`able buffer
    fn serialize(&self, buf: &mut impl Write) -> io::Result<usize>;

    fn write_string(&self, buf: &mut impl Write, string: &String) -> usize where Self : Sized {
        let bytes = string.as_bytes();
        buf.write_u16::<NetworkEndian>(bytes.len() as u16).unwrap();
        buf.write_all(&bytes).unwrap();
        2 + bytes.len()
    }

    fn write_optional_string(&self, buf: &mut impl Write, string: &Option<String>) -> usize where Self: Sized {
        buf.write_i8(string.is_some() as i8).unwrap();
        let mut bytes_written = 1;
        if let Some(str) = string {
            bytes_written += self.write_string(buf, str);
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
        if buf.read_i8().unwrap() != 0 { // if the boolean is set read the optional value
            Some(Self::read_string(buf).unwrap())
        } else { None }
    }
}