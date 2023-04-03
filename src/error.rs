#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("etcd client error")]
    EtcdError(#[from] etcd_client::Error),
    #[error("key not found")]
    NotFound,
    #[error("url parse error")]
    ParseUrlError(#[from] url::ParseError),
    #[error("date parse error")]
    ParseDateError(#[from] chrono::ParseError),
    #[error("int parse error")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("etcd utf-8 string error")]
    FromEtcdStringError(#[from] std::str::Utf8Error),
    #[error("key malformed")]
    ParseEtcdKeyError,
    #[error("error while running API")]
    RunApiError(std::io::Error),
    #[error("error while running queue dispatch")]
    RunDispatchError(std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
