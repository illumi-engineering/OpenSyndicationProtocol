use uuid::Uuid;

mod ser;
mod error;
mod de;

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
    StringBegin,
    StringEnd,
    BytesBegin,
    BytesEnd,
    NewTypeBegin,
    NewTypeEnd,
    TupleVariantBegin,
    TupleVariantBreak,
    TupleVariantEnd,
    MapBegin,
    MapBreak,
    MapEnd,
    StructVariantBegin,
    StructVariantBreak,
    StructVariantEnd,
    UnitVariantBegin,
    UnitVariantEnd,
    UnitStruct,
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
            Marker::StringBegin => 15,
            Marker::StringEnd => 16,
            Marker::BytesBegin => 17,
            Marker::BytesEnd => 18,
            Marker::NewTypeBegin => 19,
            Marker::NewTypeEnd => 20,
            Marker::TupleVariantBegin => 21,
            Marker::TupleVariantBreak => 22,
            Marker::TupleVariantEnd => 23,
            Marker::MapBegin => 24,
            Marker::MapBreak => 25,
            Marker::MapEnd => 26,
            Marker::StructVariantBegin => 27,
            Marker::StructVariantBreak => 28,
            Marker::StructVariantEnd => 29,
            Marker::UnitVariantBegin => 30,
            Marker::UnitVariantEnd=> 31,
            Marker::UnitStruct=> 32,
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
            15 => Ok(Marker::StringBegin),
            16 => Ok(Marker::StringEnd),
            17 => Ok(Marker::BytesBegin),
            18 => Ok(Marker::BytesEnd),
            19 => Ok(Marker::NewTypeBegin),
            20 => Ok(Marker::NewTypeEnd),
            21 => Ok(Marker::TupleVariantBegin),
            22 => Ok(Marker::TupleVariantBreak),
            23 => Ok(Marker::TupleVariantEnd),
            24 => Ok(Marker::MapBegin),
            25 => Ok(Marker::MapBreak),
            26 => Ok(Marker::MapEnd),
            27 => Ok(Marker::StructVariantBegin),
            28 => Ok(Marker::StructVariantBreak),
            29 => Ok(Marker::StructVariantEnd),
            30 => Ok(Marker::UnitVariantBegin),
            31 => Ok(Marker::UnitVariantEnd),
            32 => Ok(Marker::UnitStruct),
            _ => Err(error::Error::Message("Unknown ".to_string()))
        }
    }
}
