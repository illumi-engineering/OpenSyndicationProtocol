use std::net::{SocketAddr};
use tokio::net::{TcpStream};
use tokio::io::{self};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};
use crate::packet::{DeserializePacket, PacketCodec, SerializePacket};
use futures_util::sink::SinkExt;


pub struct Protocol<PacketType: DeserializePacket + SerializePacket> {
    pub read: FramedRead<OwnedReadHalf, PacketCodec<PacketType>>,
    pub write: FramedWrite<OwnedWriteHalf, PacketCodec<PacketType>>
}

impl<PacketType: DeserializePacket + SerializePacket> Protocol<PacketType> {
    /// Wrap a TcpStream with Protocol
    pub fn with_stream(stream: TcpStream) -> io::Result<Self> {
        let read_codec: PacketCodec<PacketType> = PacketCodec::new();
        let write_codec: PacketCodec<PacketType> = PacketCodec::new();
        let (read, write) = stream.into_split();
        Ok(Self {
            read: FramedRead::new(read, read_codec),
            write: FramedWrite::new(write, write_codec),
        })
    }

    /// Establish a connection, wrap stream in BufReader/Writer
    pub async fn connect(dest: SocketAddr) -> io::Result<Self> {
        let stream = TcpStream::connect(dest).await?;
        Self::with_stream(stream)
    }

    pub fn map_codec<NewPacketType, F>(self, map: F) -> Protocol<NewPacketType>
    where
        F: Fn(PacketCodec<PacketType>) -> PacketCodec<NewPacketType>,
        NewPacketType: DeserializePacket + SerializePacket,
    {
        Protocol::<NewPacketType> {
            read: self.read.map_decoder(|codec| { map(codec) }),
            write: self.write.map_encoder(|codec| { map(codec) }),
        }
    }

    /// Serialize a message to the server and write it to the TcpStream
    pub async fn send_message(&mut self, message: PacketType) -> io::Result<()> {
        self.write.send(message).await
    }

    /// Read a message from the inner TcpStream
    ///
    /// NOTE: Will block until there's data to read (or deserialize fails with io::ErrorKind::Interrupted)
    ///       so only use when a message is expected to arrive
    pub async fn read_frame(&mut self) -> io::Result<Option<PacketType::Output>> {
        Ok(self.read.next().await.unwrap().ok())
    }
}