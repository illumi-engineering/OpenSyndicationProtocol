use std::net::SocketAddr;
use std::pin::pin;
use tokio::net::{TcpStream};
use super::utils::{DeserializePacket, SerializePacket};
use tokio::io::{self, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};


pub struct Protocol {
    pub reader: BufReader<OwnedReadHalf>,
    pub writer: BufWriter<OwnedWriteHalf>,
}

impl Protocol {
    /// Wrap a TcpStream with Protocol
    pub fn with_stream(stream: TcpStream) -> io::Result<Self> {
        let (read, write) = stream.into_split();
        Ok(Self {
            reader: BufReader::new(read),
            writer: BufWriter::new(write),
        })
    }

    /// Establish a connection, wrap stream in BufReader/Writer
    pub async fn connect(dest: SocketAddr) -> io::Result<Self> {
        let stream = TcpStream::connect(dest).await?;
        Self::with_stream(stream)
    }

    /// Serialize a message to the server and write it to the TcpStream
    pub async fn send_message(&mut self, message: &impl SerializePacket) -> io::Result<()> {
        message.serialize(pin!(&mut self.writer)).await?;
        self.writer.flush().await
    }

    /// Read a message from the inner TcpStream
    ///
    /// NOTE: Will block until there's data to read (or deserialize fails with io::ErrorKind::Interrupted)
    ///       so only use when a message is expected to arrive
    pub async fn read_message<T: DeserializePacket>(&mut self) -> io::Result<T::Output> {
        T::deserialize(pin!(&mut self.reader)).await
    }
}