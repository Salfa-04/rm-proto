use crate::{calc_dji8, calc_dji16};

///
/// CRC validator abstraction for the frame protocol.
///
/// Implementations define how frame integrity is verified:
/// - CRC8 for the frame header
/// - CRC16 for the frame body
///
pub trait Validator {
    ///
    /// Calculate CRC8 over the given raw bytes.
    ///
    /// Typically used for validating the frame header.
    ///
    fn calculate_crc8(raw: &[u8]) -> u8;
    ///
    /// Calculate CRC16 over the given raw bytes.
    ///
    /// Typically used for validating the full frame
    /// (header + command + payload).
    ///
    fn calculate_crc16(raw: &[u8]) -> u16;
}

///
/// DJI protocol CRC validator.
///
/// This implementation uses DJI-compatible CRC8 and CRC16
/// algorithms for frame validation.
///
pub struct DjiValidator;

impl Validator for DjiValidator {
    fn calculate_crc8(raw: &[u8]) -> u8 {
        calc_dji8(raw)
    }

    fn calculate_crc16(raw: &[u8]) -> u16 {
        calc_dji16(raw)
    }
}
