use crate::error::EpochError;

pub struct BitWriter {
    /// Internal byte buffer
    buffer: Vec<u8>,
    /// Tracks the next bit position to write (0-7), where 0 is MSB
    bit_offset: u8,
}

impl BitWriter {
    /// Creates a new, empty `BitWriter`.
    ///
    /// The internal buffer is initialized with no capacity. It will allocate
    /// on the first call to `write_bits`.
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            bit_offset: 0,
        }
    }

    /// with_capacity creates a new `BitWriter` with a specified initial capacity for the internal buffer.
    /// This can help reduce allocations if the expected size of the data is known in advance.
    ///
    /// # Arguments
    /// * `capacity` - The initial capacity in bytes for the internal buffer.
    ///
    /// # Examples
    /// ```
    /// # use epoch_link::BitWriter;
    /// let mut writer = BitWriter::with_capacity(16);
    /// // This initializes the internal buffer with a capacity of 16 bytes.
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            bit_offset: 0,
        }
    }

    /// Appends a specific number of bits from a `u64` value into the internal buffer.
    ///
    /// # Arguments
    /// * `value` - The source data. Only the `bit_count` least significant bits are used.
    /// * `bit_count` - The number of bits to write (range: 1-64).
    ///
    /// # Layout
    /// Bits are packed from **Most Significant Bit (MSB) to Least Significant Bit (LSB)**
    /// of the current byte. If a value exceeds the current byte's capacity, it overflows
    /// into the MSB of the next byte.
    ///
    /// # Examples
    /// If the buffer is empty and we write `0b101` (3 bits):
    /// The first byte will look like `10100000`.
    ///
    /// Return Results:
    /// * `Ok(())` on success.
    /// * `Err(EpochError::InvalidBitCount)` if `bit_count` is not in the range 1-64.
    pub fn write_bits(&mut self, mut value: u64, mut bit_count: u8) -> Result<(), EpochError> {
        if bit_count > 64 {
            return Err(EpochError::InvalidBitCount(bit_count));
        }

        if bit_count == 0 {
            return Ok(());
        }

        // 1. Mask the value to ensure we only have the bits we intend to write.
        // This handles cases where a user passes a large u64 but only wants 4 bits.
        if bit_count < 64 {
            value &= (1 << bit_count) - 1;
        }

        while bit_count > 0 {
            // 2. If we are at a byte boundary (offset 0), push a new empty byte.
            if self.bit_offset == 0 {
                self.buffer.push(0);
            }

            // 3. Calculate how much room is left in the current (last) byte.
            let space_in_current_byte = 8 - self.bit_offset;

            // 4. How many bits are we actually going to write in this specific iteration?
            let bits_to_write = std::cmp::min(bit_count, space_in_current_byte);

            // 5. Shift the value to align it with the "gap" in the current byte.
            // We fill from MSB to LSB.
            let shift = space_in_current_byte - bits_to_write;
            let mask = (value >> (bit_count - bits_to_write)) as u8;

            // 6. Access the last byte and OR the bits in.
            if let Some(last_byte) = self.buffer.last_mut() {
                *last_byte |= mask << shift;
            }

            // 7. Update tracking variables.
            bit_count -= bits_to_write;
            self.bit_offset = (self.bit_offset + bits_to_write) % 8;
        }

        Ok(())
    }

    /// Returns a reference to the underlying byte slice.
    ///
    /// This method allows for reading the packed data (e.g., for CRC calculation)
    /// without consuming the writer. Note that if the last byte is partially
    /// filled, the remaining bits are trailing zeros.
    pub fn get_bytes(&self) -> &[u8] {
        &self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::EpochError;

    #[test]
    fn test_single_byte_alignment() -> Result<(), EpochError> {
        let mut writer = BitWriter::new();
        // Write 0b1010 (4 bits)
        writer.write_bits(0xA, 4)?;
        // Current state: [1010_0000] (0xA0)

        // Write 0b1111 (4 bits)
        writer.write_bits(0xF, 4)?;
        // Current state: [1010_1111] (0xAF)

        assert_eq!(writer.get_bytes(), vec![0xAF]);
        Ok(())
    }

    #[test]
    fn test_cross_byte_boundary() -> Result<(), EpochError> {
        let mut writer = BitWriter::new();
        // Write 6 bits of 1s
        writer.write_bits(0x3F, 6)?; // [111111_00]

        // Write 4 bits of 0b1010
        // 2 bits should fill the first byte, 2 bits should go to the next
        writer.write_bits(0xA, 4)?;

        let bytes = writer.get_bytes();
        // Byte 0: 111111 + 10 = 11111110 (0xFE)
        // Byte 1: 10 + 000000 = 10000000 (0x80)
        assert_eq!(bytes, vec![0xFE, 0x80]);
        Ok(())
    }

    #[test]
    fn test_large_value_span() -> Result<(), EpochError> {
        let mut writer = BitWriter::new();
        // Write a 17-bit value
        // 0x1FFFF = 1_1111_1111_1111_1111
        writer.write_bits(0x1FFFF, 17)?;

        let bytes = writer.get_bytes();
        assert_eq!(bytes.len(), 3);
        assert_eq!(bytes[0], 0xFF);
        assert_eq!(bytes[1], 0xFF);
        assert_eq!(bytes[2], 0x80); // The 17th bit at the MSB
        Ok(())
    }

    #[test]
    fn test_zero_bit_count() -> Result<(), EpochError> {
        let mut writer = BitWriter::new();

        // Writing 0 bits should be a "no-op" and return Ok
        writer.write_bits(0xFF, 0)?;

        assert_eq!(writer.get_bytes().len(), 0);
        Ok(())
    }

    #[test]
    fn test_invalid_bit_count() {
        let mut writer = BitWriter::new();
        let result = writer.write_bits(1, 128);
        assert!(
            matches!(result, Err(EpochError::InvalidBitCount(128))),
            "Expected InvalidBitCount(128), but got {:?}",
            result
        )
    }

    #[test]
    fn test_sequential_fills() -> Result<(), EpochError> {
        let mut writer = BitWriter::new();
        // Write 1 bit 8 times
        for _ in 0..8 {
            writer.write_bits(1, 1)?;
        }
        assert_eq!(writer.get_bytes(), vec![0xFF]);
        Ok(())
    }
}
