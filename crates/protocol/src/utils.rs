use std::pin::Pin;
// use tokio_byteorder::{NetworkEndian, AsyncReadBytesExt, AsyncWriteBytesExt};
use uuid::Uuid;
use tokio::io::{self, AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};


pub trait SerializePacket {
    /// Serialize to a `Write`able buffer
    async fn serialize(&self, buf: Pin<&mut impl AsyncWrite>) -> io::Result<usize>;

    async fn write_string(&self, mut buf: Pin<&mut impl AsyncWrite>, string: &String) -> io::Result<usize> where Self : Sized {
        let bytes = string.as_bytes();
        buf.write_u16(bytes.len() as u16).await?;
        buf.write_all(&bytes).await?;
        Ok(2 + bytes.len()) // u16 = 2 bytes
    }

    async fn write_optional_string(&self, mut buf: Pin<&mut impl AsyncWrite>, string: &Option<String>) -> io::Result<usize> where Self: Sized {
        buf.write_u8(string.is_some() as u8).await?;
        let mut bytes_written = 1;
        if let Some(str) = string {
            bytes_written += self.write_string(buf, str).await?;
        }
        Ok(bytes_written)
    }

    async fn write_uuid(&self, mut buf: Pin<&mut impl AsyncWrite>, uuid: &Uuid) -> io::Result<usize> where Self: Sized {
        buf.write_u128(uuid.as_u128()).await?;
        Ok(16) // u128 is 16 bytes
    }

    async fn write_optional_uuid(&self, mut buf: Pin<&mut impl AsyncWrite>, uuid: &Option<Uuid>) -> io::Result<usize> where Self: Sized {
        buf.write_u8(uuid.is_some() as u8).await?;
        let mut bytes_written = 1;
        if let Some(id) = uuid {
            bytes_written += self.write_uuid(buf, id).await?;
        }
        Ok(bytes_written)
    }
}

/// Trait for something that can be converted from bytes (&[u8])
pub trait DeserializePacket {
    /// The type that this deserializes to
    type Output;

    /// Deserialize from a `Read`able buffer
    async fn deserialize(buf: Pin<&mut impl AsyncRead>) -> io::Result<Self::Output>;

    /// From a given readable buffer (TcpStream), read the next length (u16) and extract the string bytes ([u8])
    async fn read_string(mut buf: Pin<&mut impl AsyncRead>) -> io::Result<String> {
        let length = buf.read_u16().await?;

        // Given the length of our string, only read in that quantity of bytes
        let mut bytes = vec![0u8; length as usize];
        buf.read_exact(&mut bytes).await?;

        // And attempt to decode it as UTF8
        String::from_utf8(bytes).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid utf8"))
    }

    async fn read_optional_string(mut buf: Pin<&mut impl AsyncRead>) -> io::Result<Option<String>> {
        Ok(if buf.read_u8().await? != 0 { // if the boolean is set read the optional value
            Some(Self::read_string(buf).await?)
        } else { None })
    }

    async fn read_uuid(mut buf: Pin<&mut impl AsyncRead>) -> io::Result<Uuid> {
        Ok(Uuid::from_u128(buf.read_u128().await?))
    }

    async fn read_optional_uuid(mut buf: Pin<&mut impl AsyncRead>) -> io::Result<Option<Uuid>> {
        Ok(if buf.read_u8().await? != 0 { // if the boolean is set read the optional value
            Some(Self::read_uuid(buf).await?)
        } else { None })
    }
}