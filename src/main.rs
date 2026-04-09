use epoch_link::BitWriter;
use epoch_link::EpochError;

fn main() -> Result<(), EpochError> {
    let mut writer = BitWriter::new();
    writer.write_bits(4095, 12)?;

    let bytes = writer.get_bytes();
    assert_eq!(bytes.len(), 2);
    assert_eq!(bytes[0], 0xFF);
    assert_eq!(bytes[1], 0xF0);

    writer.write_bits(17, 5)?;
    let bytes = writer.get_bytes();
    assert_eq!(bytes.len(), 3);
    assert_eq!(bytes[0], 0xFF);
    assert_eq!(bytes[1], 0xF8);
    assert_eq!(bytes[2], 0x80);

    writer.write_bits(1, 4)?;
    let bytes = writer.get_bytes();
    assert_eq!(bytes.len(), 3);
    assert_eq!(bytes[0], 0xFF);
    assert_eq!(bytes[1], 0xF8);
    assert_eq!(bytes[2], 0x88);

    println!("Hello, world!");
    Ok(())
}
