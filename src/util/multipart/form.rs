use rand::{distributions::Alphanumeric, Rng};
use serde::Serialize;

use crate::util::multipart::FormSerializer;

pub(crate) struct Form {
    pub(super) bytes: Vec<u8>,
    pub(super) boundary: [u8; 16],
}

impl Form {
    pub(super) const BOUNDARY_TERMINATOR: &'static [u8; 2] = b"--";
    pub(super) const NEWLINE: &'static [u8; 2] = b"\r\n";

    pub fn serialize<T: Serialize>(value: &T) -> Self {
        let mut serializer = FormSerializer::new();

        // The error type is infallible
        value.serialize(&mut serializer).unwrap();

        serializer.form
    }

    pub fn new() -> Self {
        let mut boundary = [0; 16];
        let mut rng = rand::thread_rng();

        boundary
            .iter_mut()
            .for_each(|value| *value = rng.sample(Alphanumeric));

        let mut bytes = Vec::with_capacity(1024);
        bytes.extend_from_slice(Self::BOUNDARY_TERMINATOR);
        bytes.extend_from_slice(&boundary);

        Self { bytes, boundary }
    }

    pub fn build(mut self) -> Vec<u8> {
        self.bytes.extend_from_slice(Self::BOUNDARY_TERMINATOR);

        self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len() + Self::BOUNDARY_TERMINATOR.len()
    }

    pub fn push_text<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        self.write_field_headers(key.as_ref(), false);
        self.bytes.extend_from_slice(value.as_ref());

        self.bytes.extend_from_slice(Self::NEWLINE);
        self.bytes.extend_from_slice(Self::BOUNDARY_TERMINATOR);
        self.bytes.extend_from_slice(&self.boundary);

        self
    }

    pub fn push_replay<K>(&mut self, key: K, replay: &[u8]) -> &mut Self
    where
        K: AsRef<[u8]>,
    {
        self.write_field_headers(key.as_ref(), true);
        self.bytes.extend_from_slice(replay);

        self.bytes.extend_from_slice(Self::NEWLINE);
        self.bytes.extend_from_slice(Self::BOUNDARY_TERMINATOR);
        self.bytes.extend_from_slice(&self.boundary);

        self
    }

    pub fn content_type(&self) -> Vec<u8> {
        const NAME: &str = "multipart/form-data; boundary=";

        let mut content_type = Vec::with_capacity(NAME.len() + self.boundary.len());
        content_type.extend_from_slice(NAME.as_bytes());
        content_type.extend_from_slice(&self.boundary);

        content_type
    }

    pub(super) fn write_field_headers(&mut self, name: &[u8], with_replay: bool) {
        self.bytes.extend_from_slice(Self::NEWLINE);
        self.bytes
            .extend_from_slice(b"Content-Disposition: form-data; name=\"");
        self.bytes.extend_from_slice(name);
        self.bytes.extend_from_slice(b"\"");

        if with_replay {
            self.bytes.extend_from_slice(b"; filename=\"replay.osr\"");
        }

        self.bytes.extend_from_slice(Self::NEWLINE);
        self.bytes.extend_from_slice(Self::NEWLINE);
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8 as str_from_utf8;

    use super::Form;

    #[test]
    fn empty() {
        let form = Form::new();

        let expect = format!("--{}--", str_from_utf8(&form.boundary).unwrap());

        let form = String::from_utf8(form.build()).unwrap();

        assert_eq!(form, expect);
    }

    #[test]
    fn filled() {
        let mut form = Form::new();

        form.push_text("key1", "value1")
            .push_text("key2", "value2")
            .push_replay("key3", b"replay data");

        let boundary = str_from_utf8(&form.boundary).unwrap();

        let expect = format!(
            "--{boundary}\r\n\
            Content-Disposition: form-data; name=\"key1\"\r\n\
            \r\n\
            value1\r\n\
            --{boundary}\r\n\
            Content-Disposition: form-data; name=\"key2\"\r\n\
            \r\n\
            value2\r\n\
            --{boundary}\r\n\
            Content-Disposition: form-data; name=\"key3\"; filename=\"replay.osr\"\r\n\
            \r\n\
            replay data\r\n\
            --{boundary}--"
        );

        let form = String::from_utf8(form.build()).unwrap();

        assert_eq!(form, expect);
    }
}
