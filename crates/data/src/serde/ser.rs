use bytes::{BufMut, BytesMut};
use serde::{Serialize, ser};
use crate::ser::DataSerializer;
use crate::serde::error::{Error, Result};

pub fn to_bytes<'ser, T>(value: &T) -> Result<&'ser mut BytesMut>
where
    T: Serialize,
{
    let mut serializer = DataSerializer::<T>::new();
    value.serialize(&mut serializer)?;
    Ok(&mut serializer.output)
}

impl<'a, Data> ser::Serializer for &'a mut DataSerializer<T> where Data : Serialize {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ();
    type SerializeTuple = ();
    type SerializeTupleStruct = ();
    type SerializeTupleVariant = ();
    type SerializeMap = ();
    type SerializeStruct = ();
    type SerializeStructVariant = ();

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output.put_u8(v as u8)?
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.output.put_i8(v)?
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.output.put_i16(v)?
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.output.put_i32(v)?
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.output.put_i64(v)?
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        self.output.put_i128(v)?
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.output.put_u8(v)?
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.output.put_u16(v)?
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.output.put_u32(v)?
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output.put_u64(v)?
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        self.output.put_u128(v)?
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.output.put_f32(v)?
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output.put_f64(v)?
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.output.put_u8(v as u8)?
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        let bytes = v.as_bytes();
        self.output.put_u16(bytes.len() as u16);
        self.output.put_slice(bytes);
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        todo!()
    }

    fn serialize_none(self) -> Result<()> {
        todo!()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize
    {
        todo!()
    }

    fn serialize_unit(self) -> Result<()> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        todo!()
    }

    fn serialize_unit_variant(self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<()> {
        todo!()
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize
    {
        todo!()
    }

    fn serialize_newtype_variant<T>(self, name: &'static str, variant_index: u32, variant: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        todo!()
    }

    fn serialize_struct_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}