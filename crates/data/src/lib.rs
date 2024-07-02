use uuid::Uuid;

#[cfg(feature = "serde")]
pub mod serde;
mod ser;

pub struct DataType<T> {
    id: Uuid
}