pub mod registry;

use bincode::{Decode, Encode};
use bincode::error::{DecodeError, EncodeError};

use bytes::{Bytes, BytesMut};

use downcast_rs::{Downcast, impl_downcast};

use uuid::Uuid;

pub trait Data : DataClone + Send {}

trait DataClone {
    fn clone_box(&self) -> Box<dyn Data>;
}

impl<TData> DataClone for TData
where
    TData: 'static + Data + Clone,
{
    fn clone_box(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Data> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait DataMarshaller : Downcast + DataMarshallerClone + Send {
    type DataType : Data + Clone + ?Sized;

    fn get_id_static() -> Uuid where Self : Sized;

    fn get_id(&self) -> Uuid where Self : Sized {
        Self::get_id_static()
    }

    fn decode_from_bytes(buf: &Bytes) -> Result<(Self::DataType, usize), DecodeError>
    where
        Self : Sized,
        Self::DataType : Decode,
    {
        let config = bincode::config::standard();
        let res = bincode::decode_from_slice(buf, config)?;
        Ok(res)
    }

    fn encode_to_bytes(buf: &mut BytesMut, obj: Self::DataType) -> Result<usize, EncodeError>
    where
        Self : Sized,
        Self::DataType : Encode,
    {
        let config = bincode::config::standard();
        let len = bincode::encode_into_slice(obj, buf, config)?;
        Ok(len)
    }
}
impl_downcast!(DataMarshaller assoc DataType where DataType : Data + Clone + ?Sized);

trait DataMarshallerClone {
    fn clone_box(&self) -> Box<dyn DataMarshaller<DataType=Box<dyn Data>>>;
}

impl<TMarshaller> DataMarshallerClone for TMarshaller
where
    TMarshaller: 'static + DataMarshaller<DataType = Box<(dyn Data + 'static)>> + Clone + ?Sized,
{
    fn clone_box(&self) -> Box<dyn DataMarshaller<DataType=Box<dyn Data>>> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn DataMarshaller<DataType=Box<dyn Data>>>
{
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
