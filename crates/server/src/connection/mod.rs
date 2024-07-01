use tokio::{io};
use osp_protocol::packet::{DeserializePacket, SerializePacket};
use osp_protocol::Protocol;

pub mod inbound;
pub mod outbound;

trait ConnectionUtils<InPacketType : DeserializePacket, OutPacketType : SerializePacket> {
    async fn await_frame_in(mut protocol: &Protocol<InPacketType, OutPacketType>) -> io::Result<InPacketType> {
        loop {
            if let Some(packet) = protocol.read_frame().await? {
                return Ok(packet)
            }
        }
    }
}