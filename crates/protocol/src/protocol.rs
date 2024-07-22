use std::net::{SocketAddr};

use tokio::io;
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use tokio_stream::StreamExt;

use tokio_util::codec::{FramedRead, FramedWrite};

use futures_util::SinkExt;

use crate::packet::{DeserializePacket, PacketDecoder, PacketEncoder, SerializePacket};

pub struct Protocol<InPacketType: DeserializePacket, OutPacketType : SerializePacket> {
    pub read: FramedRead<OwnedReadHalf, PacketDecoder<InPacketType>>,
    pub write: FramedWrite<OwnedWriteHalf, PacketEncoder<OutPacketType>>
}

impl<InPacketType, OutPacketType> Protocol<InPacketType, OutPacketType>
where
    OutPacketType : SerializePacket + Send,
    InPacketType : DeserializePacket + Send,
{
    /// Wrap a [`TcpStream`] with Protocol
    pub fn with_stream(stream: TcpStream) -> Self {
        let read_codec: PacketDecoder<InPacketType> = PacketDecoder::new();
        let write_codec: PacketEncoder<OutPacketType> = PacketEncoder::new();
        let (read, write) = stream.into_split();
        Self {
            read: FramedRead::new(read, read_codec),
            write: FramedWrite::new(write, write_codec),
        }
    }

    /// Establish a connection, and wrap the stream in a new Protocol.
    /// 
    /// # Errors
    /// If the inner [`TCPStream::connect`] errors.
    pub async fn connect(dest: SocketAddr) -> io::Result<Self> {
        let stream = TcpStream::connect(dest).await?;
        Ok(Self::with_stream(stream))
    }

    /// Change the codecs being used for incoming and outgoing packets,
    /// returning a new Protocol.
    ///
    /// Calls the underlying [`FramedWrite::map_encoder`] and Framed
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

    /// Serialize a message to the server and write it to the inner [`FramedWrite`]
    /// 
    /// # Errors
    /// If the inner [`FramedWrite::send`] errors
    pub async fn send_message(&mut self, message: OutPacketType) -> io::Result<()> {
        self.write.send(message).await
    }

    /// Read a message from the inner [`FramedRead`]
    /// 
    /// # Errors
    /// If the inner [`FramedRead::next`] errors
    pub async fn read_frame(&mut self) -> io::Result<InPacketType::Output> {
        loop {
            if let Some(packet) = self.read.next().await {
                return packet;
            }
        }
    }
}