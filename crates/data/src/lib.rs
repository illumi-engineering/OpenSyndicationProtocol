use bincode::{Decode, Encode};
use bincode::error::{DecodeError, EncodeError};
use bytes::{Bytes, BytesMut};
use uuid::Uuid;


pub trait Data {
    fn get_id() -> Uuid where Self : Sized;

    fn decode_from_bytes(buf: &Bytes) -> Result<(Self, usize), DecodeError>
    where
        Self : Decode + Sized
    {
        let config = bincode::config::standard();
        let res = bincode::decode_from_slice(buf, config)?;
        Ok(res)
    }

    fn encode_to_bytes(buf: &mut BytesMut, obj: Self) -> Result<usize, EncodeError>
    where
        Self : Encode + Sized
    {
        let config = bincode::config::standard();
        let len = bincode::encode_into_slice(obj, buf, config)?;
        Ok(len)
    }
}
