use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrbitError {
    #[error("Serialization failure: {0}")]
    Serialization(#[from] postcard::Error),

    #[error("Storage I/O failure: {0}")]
    Io(#[from] std::io::Error),

    #[error("Buffer overflow: maximum ring buffer capacity reached")]
    BufferOverflow,

    #[error("Data corruption detected: invalid header checksum")]
    ChecksumMismatch,
}