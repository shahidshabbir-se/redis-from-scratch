use bytes::Bytes;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(i64),
    Bulk(Bytes),
    Array(Vec<Frame>),
    Null,
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum Error {
    #[error("Incomplete frame: need more data")]
    Incomplete,

    #[error("Protocol error: {0}")]
    Protocol(String),
}
