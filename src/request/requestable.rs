use hyper::{body::Bytes, StatusCode};

use crate::ClientError;

pub(crate) trait Requestable {
    fn response_error(status: StatusCode, bytes: Bytes) -> ClientError;
}
