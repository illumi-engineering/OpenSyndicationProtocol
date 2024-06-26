use std::net::{SocketAddr};

use futures_util::SinkExt;

use tokio::io::{self};
use tokio::net::{TcpStream};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::packet::{DeserializePacket, PacketDecoder, PacketEncoder, SerializePacket};

pub struct Protocol<InPacketType: DeserializePacket, OutPacketType : SerializePacket> {
    pub read: FramedRead<OwnedReadHalf, PacketDecoder<InPacketType>>,
    pub write: FramedWrite<OwnedWriteHalf, PacketEncoder<OutPacketType>>
}

impl<InPacketType: DeserializePacket, OutPacketType : SerializePacket> Protocol<InPacketType, OutPacketType> {
    /// Wrap a TcpStream with Protocol
    pub fn with_stream(stream: TcpStream) -> io::Result<Self> {
        let read_codec: PacketDecoder<InPacketType> = PacketDecoder::new();
        let write_codec: PacketEncoder<OutPacketType> = PacketEncoder::new();
        let (read, write) = stream.into_split();
        Ok(Self {
            read: FramedRead::new(read, read_codec),
            write: FramedWrite::new(write, write_codec),
        })
    }

    /// Establish a connection, and wrap the stream in a new [Protocol].
    pub async fn connect(dest: SocketAddr) -> io::Result<Self> {
        let stream = TcpStream::connect(dest).await?;
        Self::with_stream(stream)
    }

    /// Change the codecs being used for incoming and outgoing packets,
    /// returning a new [Protocol].
    ///
    /// Calls the underlying [FramedWrite::map_encoder] and Framed
    pub fn map_codecs<NewInPacketType, NewOutPacketType, FnInPacket, FnOutPacket>(self, map_in: FnInPacket, map_out: FnOutPacket) -> Protocol<NewInPacketType, NewOutPacketType>
    where
        FnInPacket: FnOnce(PacketDecoder<InPacketType>) -> PacketDecoder<NewInPacketType>,
        FnOutPacket: FnOnce(PacketEncoder<OutPacketType>) -> PacketEncoder<NewOutPacketType>,
        NewInPacketType: DeserializePacket,
        NewOutPacketType: SerializePacket,
    {
        Protocol::<NewInPacketType, NewOutPacketType> {
            read: self.read.map_decoder(map_in),
            write: self.write.map_encoder(map_out),
        }
    }

    /// Serialize a message to the server and write it to the inner [FramedWrite]
    pub async fn send_message(&mut self, message: OutPacketType) -> io::Result<()> {
        self.write.send(message).await
    }

    /// Read a message from the inner [FramedRead]
    pub async fn read_frame(&mut self) -> io::Result<Option<InPacketType::Output>> {
        Ok(self.read.next().await.unwrap().ok())
    }
}