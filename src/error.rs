#[derive(Debug)]
pub enum EpochError {
    InvalidBitCount(u8),
    BufferTooSmall,
    // We'll add more as we build the protocol
}
