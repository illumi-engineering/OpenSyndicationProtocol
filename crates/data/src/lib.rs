use uuid::Uuid;

#[cfg(feature = "serde")]
pub mod serde;
mod ser;
mod error;

pub struct DataType<T> {
    id: Uuid
}

pub enum Marker {
    Unit = 0,
    SeqBegin = 1,
    SeqEnd = 2,
}

// impl From<Marker> for u8 {
//     fn from(marker: Marker) -> Self {
//         match marker {
//             Marker::OptionBegin => 1,
//             Marker::OptionEnd => 2,
//         }
//     }
// }
