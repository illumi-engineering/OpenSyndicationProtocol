use tokio::{io};
use osp_protocol::packet::{DeserializePacket, SerializePacket};
use osp_protocol::Protocol;

pub mod inbound;
pub mod outbound;
