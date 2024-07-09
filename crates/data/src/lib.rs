pub mod registry;

use std::marker::PhantomData;
use bincode::{Decode, Encode};
use bincode::error::{DecodeError, EncodeError};

use bytes::{Bytes, BytesMut};

// use downcast_rs::{Downcast, impl_downcast};

use dyn_clone::{clone_trait_object, DynClone};

use uuid::Uuid;

pub trait Data : DynClone + Send {
    fn get_id_static() -> Uuid where Self : Sized;

    fn get_id(&self) -> Uuid where Self : Sized {
        Self::get_id_static()
    }
}
clone_trait_object!(Data);

#[derive(Clone)]
pub struct DataMarshaller<TData>
where
    TData : Data + 'static
{
    _t_data: PhantomData<TData>
}

impl<TData> DataMarshaller<TData>
where
    TData : Data + 'static
{
    fn new() -> Self {
        DataMarshaller::<TData> {
            _t_data: PhantomData,
        }
    }

    pub fn get_id(&self) -> Uuid where TData : Sized {
        TData::get_id_static()
    }

    fn decode_from_bytes(buf: &Bytes) -> Result<(TData, usize), DecodeError>
    where
        Self : Sized,
        TData : Decode,
    {
        let config = bincode::config::standard();
        let res = bincode::decode_from_slice(buf, config)?;
        Ok(res)
    }

    fn encode_to_bytes(buf: &mut BytesMut, obj: TData) -> Result<usize, EncodeError>
    where
        Self : Sized,
        TData : Encode,
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
