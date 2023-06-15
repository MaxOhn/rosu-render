use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

use serde::{
    ser::{Error as SerError, Impossible, SerializeStruct},
    Serialize, Serializer,
};

use super::Form;

pub(crate) struct FormSerializer {
    pub(super) form: Form,
    float_buf: ryu::Buffer,
    int_buf: itoa::Buffer,
}

impl FormSerializer {
    pub(crate) fn new() -> Self {
        Self {
            form: Form::new(),
            float_buf: ryu::Buffer::new(),
            int_buf: itoa::Buffer::new(),
        }
    }
}

impl Serializer for &mut FormSerializer {
    type Ok = ();
    type Error = Infallible;
    type SerializeStruct = Self;

    type SerializeSeq = Impossible<(), Infallible>;
    type SerializeTuple = Impossible<(), Infallible>;
    type SerializeTupleStruct = Impossible<(), Infallible>;
    type SerializeTupleVariant = Impossible<(), Infallible>;
    type SerializeMap = Impossible<(), Infallible>;
    type SerializeStructVariant = Impossible<(), Infallible>;

    fn serialize_bool(self, b: bool) -> Result<Self::Ok, Self::Error> {
        let bytes: &[u8] = if b { b"true" } else { b"false" };

        self.serialize_bytes(bytes)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        let s = self.int_buf.format(v);
        self.form.bytes.extend_from_slice(s.as_bytes());

        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        let s = self.int_buf.format(v);
        self.form.bytes.extend_from_slice(s.as_bytes());

        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        let s = self.int_buf.format(v);
        self.form.bytes.extend_from_slice(s.as_bytes());

        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        let s = self.int_buf.format(v);
        self.form.bytes.extend_from_slice(s.as_bytes());

        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        let s = self.int_buf.format(v);
        self.form.bytes.extend_from_slice(s.as_bytes());

        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        let s = self.int_buf.format(v);
        self.form.bytes.extend_from_slice(s.as_bytes());

        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        let s = self.int_buf.format(v);
        self.form.bytes.extend_from_slice(s.as_bytes());

        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        let s = self.int_buf.format(v);
        self.form.bytes.extend_from_slice(s.as_bytes());

        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        let s = self.float_buf.format(v);
        self.form.bytes.extend_from_slice(s.as_bytes());

        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let s = self.float_buf.format(v);
        self.form.bytes.extend_from_slice(s.as_bytes());

        Ok(())
    }

    fn serialize_char(self, _: char) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_str(self, s: &str) -> Result<Self::Ok, Self::Error> {
        self.form.bytes.extend_from_slice(s.as_bytes());

        Ok(())
    }

    fn serialize_bytes(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.form.bytes.extend_from_slice(bytes);

        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_some<T: Serialize + ?Sized>(self, _: &T) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: &T,
    ) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        unimplemented!()
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        unimplemented!()
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        unimplemented!()
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        unimplemented!()
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unimplemented!()
    }

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        unimplemented!()
    }
}

impl SerializeStruct for &mut FormSerializer {
    type Ok = ();
    type Error = Infallible;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.form.write_field_headers(key.as_bytes(), false);
        value.serialize(&mut **self)?;

        self.form.bytes.extend_from_slice(Form::NEWLINE);
        self.form.bytes.extend_from_slice(Form::BOUNDARY_TERMINATOR);
        self.form.bytes.extend_from_slice(&self.form.boundary);

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct Infallible;

impl SerError for Infallible {
    fn custom<T: Display>(_: T) -> Self {
        Self
    }
}

impl StdError for Infallible {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl Display for Infallible {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        <Self as Debug>::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::RenderOptions;

    use super::*;

    #[test]
    fn test_form_serializer() {
        let _form = Form::serialize(&RenderOptions::default());
    }
}
