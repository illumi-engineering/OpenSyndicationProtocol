use bytes::{BufMut, BytesMut};
use serde::{ser, Serialize, Serializer};
use crate::error::{Error, Result};
use crate::Marker;

pub struct DataSerializer {
    pub(crate) output: BytesMut,
    pub(crate) bytes_written: usize,
}

impl DataSerializer {
    pub fn new() -> Self {
        DataSerializer {
            output: BytesMut::new(),
            bytes_written: 0,
        }
    }

    fn marker(&self, mark: Marker) -> Result<()> {
        self.serialize_u16(mark as u16)
    }
}

pub fn to_bytes<'ser, T>(value: &T) -> Result<&'ser mut BytesMut>
where
    T: Serialize,
{
    let mut serializer = DataSerializer::new();
    value.serialize(&mut serializer)?;
    Ok(&mut serializer.output)
}

impl<'a> Serializer for &'a mut DataSerializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, value: bool) -> Result<()> {
        self.output.put_u8(value as u8)?;
        self.bytes_written += 1;
        Ok(())
    }

    fn serialize_i8(self, value: i8) -> Result<()> {
        self.output.put_i8(value)?;
        self.bytes_written += 1;
        Ok(())
    }

    fn serialize_i16(self, value: i16) -> Result<()> {
        self.output.put_i16(value)?;
        self.bytes_written += 2;
        Ok(())
    }

    fn serialize_i32(self, value: i32) -> Result<()> {
        self.output.put_i32(value)?;
        self.bytes_written += 4;
        Ok(())
    }

    fn serialize_i64(self, value: i64) -> Result<()> {
        self.output.put_i64(value)?;
        self.bytes_written += 8;
        Ok(())
    }

    fn serialize_i128(self, value: i128) -> Result<()> {
        self.output.put_i128(value)?;
        self.bytes_written += 16;
        Ok(())
    }

    fn serialize_u8(self, value: u8) -> Result<()> {
        self.output.put_u8(value)?;
        self.bytes_written += 1;
        Ok(())
    }

    fn serialize_u16(self, value: u16) -> Result<()> {
        self.output.put_u16(value)?;
        self.bytes_written += 2;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.output.put_u32(v)?;
        self.bytes_written += 4;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output.put_u64(v)?;
        self.bytes_written += 8;
        Ok(())
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        self.output.put_u128(v)?;
        self.bytes_written += 16;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.output.put_f32(v)?;
        self.bytes_written += 4;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output.put_f64(v)?;
        self.bytes_written += 8;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.output.put_u8(v as u8)?;
        self.bytes_written += 1;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.marker(Marker::StringBegin)?;
        let bytes = v.as_bytes();
        self.output.put_u16(bytes.len() as u16);
        self.output.put_slice(bytes);
        self.bytes_written += bytes.len();
        self.marker(Marker::StringEnd)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.marker(Marker::StringBegin)?;
        self.output.put_u16(v.len() as u16);
        self.output.put_slice(v);
        self.bytes_written += v.len();
        self.marker(Marker::BytesEnd)
    }

    fn serialize_none(self) -> Result<()> {
        self.marker(Marker::OptBegin)?;
        self.serialize_bool(false)?; // not present
        self.serialize_unit()?;
        self.marker(Marker::OptEnd)
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize
    {
        self.marker(Marker::OptBegin)?;
        self.serialize_bool(true)?; // present
        value.serialize(self)?;
        self.marker(Marker::OptEnd)
    }

    fn serialize_unit(self) -> Result<()> {
        self.marker(Marker::Unit)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        self.marker(Marker::UnitStruct)
    }

    fn serialize_unit_variant(self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<()> {
        self.marker(Marker::UnitVariantBegin)?;
        variant_index.serialize(**self)?;
        self.marker(Marker::UnitVariantEnd)
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(self, name: &'static str, variant_index: u32, variant: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize
    {
        self.marker(Marker::NewTypeBegin)?;
        variant_index.serialize(&mut *self)?;
        value.serialize(&mut *self)?;
        self.marker(Marker::NewTypeEnd)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.marker(Marker::SeqBegin)?;
        len.serialize(**self)?;
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.marker(Marker::TupleBegin)?;
        len.serialize(**self)?;
        Ok(self)
    }

    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct> {
        self.marker(Marker::TupleStructBegin)?;
        len.serialize(**self)?;
        Ok(self)
    }

    fn serialize_tuple_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant> {
        self.marker(Marker::TupleVariantBegin)?;
        variant_index.serialize(**self)?;
        len.serialize(**self)?;
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.marker(Marker::MapBegin)?;
        len.serialize(**self)?;
        Ok(self)
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.marker(Marker::StructBegin)?;
        len.serialize(**self)?;
        Ok(self)
    }

    fn serialize_struct_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeStructVariant> {
        self.marker(Marker::StructVariantBegin)?;
        variant_index.serialize(**self)?;
        len.serialize(**self)?;
        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut DataSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize
    {
        let last_marker = Marker::try_from(&self.output[(self.output.len()-1)..][0])?;

        if last_marker != Marker::SeqBegin {
            self.marker(Marker::SeqBreak)?;
        }

        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.marker(Marker::SeqEnd)
    }
}

impl<'a> ser::SerializeTuple for &'a mut DataSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<TElement>(&mut self, value: &TElement) -> Result<()>
    where
        TElement: ?Sized + Serialize,
    {
        let last_marker = Marker::try_from(&self.output[(self.output.len()-1)..][0])?;

        if last_marker != Marker::TupleBegin {
            self.marker(Marker::TupleBreak)?;
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.marker(Marker::TupleEnd)
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut DataSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let last_marker = Marker::try_from(&self.output[(self.output.len()-1)..][0])?;

        if last_marker != Marker::TupleStructBegin {
            self.marker(Marker::TupleStructBreak)?;
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.marker(Marker::TupleStructEnd)?;
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut DataSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let last_marker = Marker::try_from(&self.output[(self.output.len()-1)..][0])?;

        if last_marker != Marker::TupleVariantBegin {
            self.marker(Marker::TupleStructBreak)?;
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.marker(Marker::TupleVariantEnd)
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a> ser::SerializeMap for &'a mut DataSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let last_marker = Marker::try_from(&self.output[(self.output.len()-1)..][0])?;

        if last_marker != Marker::MapBegin {
            self.marker(Marker::MapBreak)?;
        }

        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.marker(Marker::MapEnd)
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for &'a mut DataSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let last_marker = Marker::try_from(&self.output[(self.output.len()-1)..][0])?;

        if last_marker != Marker::StructBegin {
            self.marker(Marker::StructBreak)?;
        }

        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.marker(Marker::StructEnd)
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for &'a mut DataSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let last_marker = Marker::try_from(&self.output[(self.output.len()-1)..][0])?;

        if last_marker != Marker::StructVariantBegin {
            self.marker(Marker::StructVariantBreak)?;
        }
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.marker(Marker::StructVariantEnd)
    }
}
