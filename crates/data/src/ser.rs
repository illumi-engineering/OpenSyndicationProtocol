use bytes::{BufMut, BytesMut};
use serde::{ser, Serialize};
use crate::error::{Error, Result};
use crate::Marker;

pub struct DataSerializer<TData> where TData : Serialize {
    pub(crate) output: BytesMut,
    pub(crate) bytes_written: usize,
}

impl<TData> DataSerializer<TData> {
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
    let mut serializer = DataSerializer::<T>::new();
    value.serialize(&mut serializer)?;
    Ok(&mut serializer.output)
}

impl<'a, TData> ser::Serializer for &'a mut DataSerializer<TData> where TData : Serialize {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ();
    type SerializeTuple = ();
    type SerializeTupleStruct = ();
    type SerializeTupleVariant = ();
    type SerializeMap = ();
    type SerializeStruct = ();
    type SerializeStructVariant = ();

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
        let bytes = v.as_bytes();
        self.output.put_u16(bytes.len() as u16);
        self.output.put_slice(bytes);
        self.bytes_written += bytes.len();
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.output.put_u16(v.len() as u16);
        self.output.put_slice(v);
        self.bytes_written += v.len();
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize
    {
        todo!()
    }

    fn serialize_unit(self) -> Result<()> {
        self.marker(Marker::Unit)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<()> {
        self.serialize_u32(variant_index)
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

impl<'a, TData> ser::SerializeSeq for &'a mut DataSerializer<TData>

trait SerializeData<TData> where TData: SerializeData<TData> {
    fn serialize(serializer: DataSerializer<TData, dyn SerializeData<TData>>) -> io::Result<usize>;
}
