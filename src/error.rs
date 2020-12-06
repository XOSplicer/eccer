#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("etcd client error")]
    EtcdError(#[from] etcd_client::Error),
    #[error("key not found")]
    NotFound,
    #[error("url parse error")]
    ParseUrlError(#[from] url::ParseError),
    #[error("etcd utf-8 string error")]
    FromEtcdStringError(#[from] std::str::Utf8Error),
    #[error("key malformed")]
    ParseEtcdKeyError,
}