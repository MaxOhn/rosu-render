use hyper::{body::Bytes, StatusCode};

use crate::Error;

pub(crate) trait Requestable {
    fn response_error(status: StatusCode, bytes: Bytes) -> Error;
}
