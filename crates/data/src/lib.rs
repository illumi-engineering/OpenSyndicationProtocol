pub mod registry;

use bincode::{Decode, Encode};
use bincode::error::{DecodeError, EncodeError};

use bytes::{Bytes, BytesMut};

// use downcast_rs::{Downcast, impl_downcast};

// use dyn_clone::{clone_trait_object, DynClone};

use uuid::Uuid;

/// Base type for all OSP data objects
///
/// ## Example implementation
/// ```rust
/// use bincode::{Decode, Encode};
/// use osp_data::impl_data;
///
/// #[derive(Encode, Decode, Clone)]
/// pub struct MyData {
///     test_val: bool,
/// }
///
/// impl_data!(MyData, "995f6806-7c36-4e27-ab03-a422952287b6");
/// ```
pub trait Data : Send {
    fn get_id_static() -> Uuid where Self : Sized;

    fn get_id(&self) -> Uuid where Self : Sized {
        Self::get_id_static()
    }
}

/// A marshaller type that contains encode/decode methods for writing [Data] to
/// a buffer, and some associated values
#[derive(Clone)]
pub struct DataMarshaller {
    /// The type [Uuid] of the type this marshaller is assigned to
    id: Uuid,
}

impl DataMarshaller {
    /// Create a new data marshaller with the type [Uuid] `id`
    fn new(id: Uuid) -> Self {
        DataMarshaller {
            id,
        }
    }

    /// Get the type [Uuid] of this marshaller
    pub fn get_id(&self) -> Uuid {
        self.id
    }

    /// Decode a [TData] off a buffer
    pub fn decode_from_bytes<TData>(self, buf: &Bytes) -> Result<(TData, usize), DecodeError>
    where
        TData : Data + Decode,
    {
        let config = bincode::config::standard();
        let res = bincode::decode_from_slice(buf, config)?;
        Ok(res)
    }

    /// Encode a [TData] onto a buffer
    pub fn encode_to_bytes<TData>(self, buf: &mut BytesMut, obj: TData) -> Result<usize, EncodeError>
    where
        TData : Data + Encode,
    {
        let config = bincode::config::standard();
        let len = bincode::encode_into_slice(obj, buf, config)?;
        Ok(len)
    }
}

/// Implement data methods more easily
///
/// **Usage:** (Given some concrete type `YourType`) `impl_data!(YourType, "your-type-uuid");`
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
