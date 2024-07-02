use uuid::Uuid;

mod ser;
mod error;

pub struct DataType<T> {
    id: Uuid
}

#[derive(PartialEq)]
pub enum Marker {
    Unit,
    SeqBegin,
    SeqBreak,
    SeqEnd,
    OptBegin,
    OptEnd,
    StructBegin,
    StructBreak,
    StructEnd,
    TupleBegin,
    TupleBreak,
    TupleEnd,
    TupleStructBegin,
    TupleStructBreak,
    TupleStructEnd,
}

impl From<Marker> for u8 {
    fn from(value: Marker) -> Self {
        match value {
            Marker::Unit => 0,
            Marker::SeqBegin => 1,
            Marker::SeqBreak => 2,
            Marker::SeqEnd => 3,
            Marker::OptBegin => 4,
            Marker::OptEnd => 5,
            Marker::StructBegin => 6,
            Marker::StructBreak => 7,
            Marker::StructEnd => 8,
            Marker::TupleBegin => 9,
            Marker::TupleBreak => 10,
            Marker::TupleEnd => 11,
            Marker::TupleStructBegin => 12,
            Marker::TupleStructBreak => 13,
            Marker::TupleStructEnd => 14,
        }
    }
}

impl TryFrom<u8> for Marker {
    type Error = error::Error;

    fn try_from(value: u8) -> error::Result<Self> {
        match value {
            0 => Ok(Marker::Unit),
            1 => Ok(Marker::SeqBegin),
            2 => Ok(Marker::SeqBreak),
            3 => Ok(Marker::SeqEnd),
            4 => Ok(Marker::OptBegin),
            5 => Ok(Marker::OptEnd),
            6 => Ok(Marker::StructBegin),
            7 => Ok(Marker::StructBreak),
            8 => Ok(Marker::StructEnd),
            9 => Ok(Marker::TupleBegin),
            10 => Ok(Marker::TupleBreak),
            11 => Ok(Marker::TupleEnd),
            12 => Ok(Marker::TupleStructBegin),
            13 => Ok(Marker::TupleStructBreak),
            14 => Ok(Marker::TupleStructEnd),
            _ => Err(error::Error::Message("Unknown ".to_string()))
        }
    }
}
