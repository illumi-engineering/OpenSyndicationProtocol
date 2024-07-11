pub mod registry;

use bincode::{Decode, Encode};
use bincode::error::{DecodeError, EncodeError};

use bytes::{Bytes, BytesMut};

// use downcast_rs::{Downcast, impl_downcast};

use dyn_clone::{clone_trait_object, DynClone};

use uuid::Uuid;

pub trait Data : Send {
    fn get_id_static() -> Uuid where Self : Sized;

    fn get_id(&self) -> Uuid where Self : Sized {
        Self::get_id_static()
    }
}

#[derive(Clone)]
pub struct DataMarshaller {
    id: Uuid,
}

impl DataMarshaller {
    fn new(id: Uuid) -> Self {
        DataMarshaller {
            id,
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn decode_from_bytes<TData>(buf: &Bytes) -> Result<(TData, usize), DecodeError>
    where
        Self : Sized,
        TData : Data + Decode,
    {
        let config = bincode::config::standard();
        let res = bincode::decode_from_slice(buf, config)?;
        Ok(res)
    }

    pub fn encode_to_bytes<TData>(buf: &mut BytesMut, obj: TData) -> Result<usize, EncodeError>
    where
        Self : Sized,
        TData : Data + Encode,
    {
        let config = bincode::config::standard();
        let len = bincode::encode_into_slice(obj, buf, config)?;
        Ok(len)
    }
}

#[macro_export]
macro_rules! impl_data {
    ($t:ident, $id:expr) => {
        impl Data for $t {
            fn get_id_static() -> Uuid
            where
                Self: Sized
            {
                Uuid::from_str($id).unwrap()
            }
        }
    };
}
