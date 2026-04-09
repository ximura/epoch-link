// Modules
pub mod codec;
pub mod error;

// Re-export for convenience
pub use error::EpochError;

pub use codec::bit_reader::BitReader;
pub use codec::bit_writer::BitWriter;
