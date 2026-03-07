//! Frame Encoding and Decoding.

use crate::PackError;
use crate::{DjiValidator, UnPackError, Validator};
use crate::{ImplMarshal, ImplUnMarshal, MarshalerError};
use core::marker::PhantomData;

/// Start of Frame Byte
const SOF: u8 = 0xA5;

/// Size of the frame header (SOF + length + sequence + CRC8).
const HEAD_SIZE: usize = 5;
/// Size of the command ID field.
const CMDID_SIZE: usize = size_of::<u16>();
/// Size of the tail CRC field.
const TAIL_SIZE: usize = size_of::<u16>();

/// Frame encoder and decoder.
///
/// [`Messager`] packs typed messages into binary frames and unpacks
/// validated frames from raw bytes. It performs no I/O and allocates
/// no memory.
///
/// Generic over [`Validator`] to allow custom CRC implementations.
/// The default validator is [`DjiValidator`].
///
/// The internal sequence counter starts at the value passed to [`new`](Self::new)
/// and increments by one after each successful [`pack`](Self::pack) call,
/// wrapping on overflow.
///
/// # Frame Layout
///
/// ```text
/// +--------+--------+--------+--------+--------+---------+--------+
/// |  SOF   |  LEN   |  SEQ   |  CRC8  | CMD_ID |  DATA   | CRC16  |
/// +--------+--------+--------+--------+--------+---------+--------+
/// | 1 byte | 2 byte | 1 byte | 1 byte | 2 byte | N bytes | 2 byte |
/// +--------+--------+--------+--------+--------+---------+--------+
/// ```
///
/// - `SOF`: start-of-frame marker (`0xA5`)
/// - `LEN`: payload length, little-endian `u16`
/// - `SEQ`: sequence number, `u8`
/// - `CRC8`: checksum over `[SOF, LEN, SEQ]`
/// - `CMD_ID`: command identifier, little-endian `u16`
/// - `DATA`: payload bytes (N = [`crate::Marshaler::PAYLOAD_SIZE`])
/// - `CRC16`: checksum over the entire frame preceding this field
///
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Messager<V: Validator = DjiValidator> {
    /// Current frame sequence number.
    sequence: u8,
    /// Marker for the validator type.
    _marker: PhantomData<V>,
}

impl<V: Validator> Messager<V> {
    /// Creates a new [`Messager`] with the given initial sequence number.
    ///
    /// The sequence number is embedded in each packed frame header and
    /// increments automatically on each successful [`pack`](Self::pack) call.
    pub const fn new(seq: u8) -> Self {
        Self {
            sequence: seq,
            _marker: PhantomData,
        }
    }

    /// Encode a message into a binary frame.
    ///
    /// Serializes `msg` and writes the complete frame — header, command ID,
    /// payload, and CRC — into `dst`. The payload is written directly into
    /// `dst` with no intermediate buffer.
    ///
    /// On success, returns the total number of bytes written.
    ///
    /// The sequence counter increments by one after each successful call.
    ///
    /// # Errors
    ///
    /// - [`PackError::BufferTooSmall`] — `dst` is smaller than the full frame.
    ///   `need` is the minimum required size in bytes.
    /// - [`PackError::InvalidPayloadSize`] — the marshaler returned a byte
    ///   count that does not equal [`crate::Marshaler::PAYLOAD_SIZE`].
    /// - [`PackError::InputTooLarge`] — the marshaler returned a byte count
    ///   exceeding `u16::MAX`.
    /// - [`PackError::MarshalerError`] — the marshaler itself failed.
    pub fn pack<M: ImplMarshal>(&mut self, msg: &M, dst: &mut [u8]) -> Result<usize, PackError> {
        let mut cursor: usize = 0;

        let payload_offset = HEAD_SIZE + CMDID_SIZE;
        let payload_size = M::PAYLOAD_SIZE as usize;

        // Ensure space for the entire frame before writing anything.
        let total = payload_offset + payload_size + TAIL_SIZE;
        if dst.len() < total {
            return Err(PackError::BufferTooSmall { need: total });
        }

        // Serialize payload directly into the destination buffer.
        let size = msg.marshal(&mut dst[payload_offset..payload_offset + payload_size])?;

        // Validate payload length.
        if size > u16::MAX as usize {
            return Err(PackError::InputTooLarge {
                max: u16::MAX as usize,
            });
        } else if size != payload_size {
            return Err(PackError::InvalidPayloadSize {
                expected: M::PAYLOAD_SIZE as usize,
                found: size,
            });
        }

        // Prepare Header
        let cmd_id = M::CMD_ID;
        let sequence = self.sequence;

        // Build frame header.
        let header: [u8; HEAD_SIZE] = {
            let mut temp = [0; _];
            let size_bytes = (payload_size as u16).to_le_bytes();
            temp[0] = SOF;
            temp[1] = size_bytes[0];
            temp[2] = size_bytes[1];
            temp[3] = sequence;
            temp[4] = V::calculate_crc8(&temp[..4]);
            temp
        };

        // Write header.
        dst[cursor..cursor + HEAD_SIZE].copy_from_slice(&header);
        cursor += HEAD_SIZE;

        // Write command ID.
        dst[cursor..cursor + CMDID_SIZE].copy_from_slice(&cmd_id.to_le_bytes());
        cursor += CMDID_SIZE;

        // Skip over payload (already written).
        cursor += payload_size;

        // Write frame CRC.
        let crc: u16 = V::calculate_crc16(&dst[..cursor]);
        dst[cursor..cursor + TAIL_SIZE].copy_from_slice(&crc.to_le_bytes());
        cursor += TAIL_SIZE;

        // Advance sequence number.
        self.sequence = self.sequence.wrapping_add(1);

        Ok(cursor)
    }

    /// Parse and validate one frame from raw bytes.
    ///
    /// Reads from the start of `src` and performs these checks in order:
    ///
    /// 1. Start-of-frame marker (`0xA5`)
    /// 2. Header CRC8
    /// 3. Frame CRC16
    ///
    /// On success, returns a [`RawFrame`] whose payload borrows from `src`,
    /// and the number of bytes consumed.
    ///
    /// The payload in the returned [`RawFrame`] is untyped. Use
    /// [`RawFrame::unmarshal`] to decode it, or call [`unmarshal`](Self::unmarshal)
    /// instead of this method to do both steps at once.
    ///
    /// # Errors
    ///
    /// - [`UnPackError::ReSync`] — `src` does not start with SOF.
    ///   `skip` is the offset of the next SOF byte; discard that many bytes
    ///   and retry.
    /// - [`UnPackError::MissingHeader`] — no SOF byte found anywhere in `src`.
    ///   `skip` equals `src.len()`; discard the entire buffer and wait for
    ///   more data.
    /// - [`UnPackError::UnexpectedEnd`] — the frame is truncated.
    ///   Keep the existing bytes and append more data before retrying.
    /// - [`UnPackError::InvalidChecksum`] — CRC validation failed.
    ///   Call [`UnPackError::skip`] to determine how many bytes to discard.
    pub fn unpack<'t>(&self, src: &'t [u8]) -> Result<(RawFrame<'t>, usize), UnPackError> {
        let mut cursor = 0;

        // Locate start-of-frame.
        if !src.starts_with(&[SOF]) {
            if let Some(start) = src.iter().position(|&x| SOF == x) {
                return Err(UnPackError::ReSync { skip: start });
            } else {
                return Err(UnPackError::MissingHeader { skip: src.len() });
            }
        }

        // Read header.
        let Some(header) = src.get(cursor..cursor + HEAD_SIZE) else {
            return Err(UnPackError::UnexpectedEnd { read: src.len() });
        };
        cursor += HEAD_SIZE;

        // Validate header and extract metadata.
        let (length, sequence) = {
            let (header_bytes, crc) = (&header[..4], header[4]);
            if V::calculate_crc8(header_bytes) != crc {
                return Err(UnPackError::InvalidChecksum { at: cursor });
            }

            // `header_bytes[0]` is the SOF byte, which we have already validated.
            let length = u16::from_le_bytes([header_bytes[1], header_bytes[2]]);
            let sequence = header_bytes[3];
            (length as usize, sequence)
        };

        // Read command ID.
        let Some(cmd) = src.get(cursor..cursor + CMDID_SIZE) else {
            return Err(UnPackError::UnexpectedEnd { read: src.len() });
        };
        cursor += CMDID_SIZE;

        // Read payload.
        let Some(payload) = src.get(cursor..cursor + length) else {
            return Err(UnPackError::UnexpectedEnd { read: src.len() });
        };
        cursor += length;

        // Get the raw data for CRC calculation
        // Note: `cursor` is within bounds due to previous checks
        let frame_bytes = &src[..cursor];

        // Read and validate tail CRC.
        let Some(tail) = src.get(cursor..cursor + TAIL_SIZE) else {
            return Err(UnPackError::UnexpectedEnd { read: src.len() });
        };
        cursor += TAIL_SIZE;

        // Note: `tail` has a Fixed Length of 2
        let crc = u16::from_le_bytes([tail[0], tail[1]]);

        // Validate CRC
        if V::calculate_crc16(frame_bytes) != crc {
            return Err(UnPackError::InvalidChecksum { at: cursor });
        }

        // Parse Cmd ID
        let cmd_id = u16::from_le_bytes([cmd[0], cmd[1]]);

        // Construct Payload
        Ok((
            RawFrame {
                cmd_id,
                sequence,
                payload,
            },
            cursor,
        ))
    }

    /// Parse a frame and decode its payload into a typed message.
    ///
    /// Combines [`unpack`](Self::unpack) and [`RawFrame::unmarshal`] in one call.
    ///
    /// On success, returns the decoded message and the number of bytes consumed.
    ///
    /// # Errors
    ///
    /// - Returns [`UnPackError`] for any framing or CRC failure (see [`unpack`](Self::unpack)).
    /// - Returns [`UnPackError::MarshalerError`] if command ID or payload size
    /// does not match `M`, or if [`M::unmarshal`](crate::Marshaler::unmarshal) fails.
    pub fn unmarshal<'t, M: ImplUnMarshal>(
        &self,
        src: &'t [u8],
    ) -> Result<(M, usize), UnPackError> {
        let (frame, cursor) = self.unpack(src)?;
        Ok((frame.unmarshal::<M>()?, cursor))
    }
}

/// A validated, undecoded frame.
///
/// Produced by [`Messager::unpack`]. The frame has passed all structural
/// and CRC checks, but the payload is still raw bytes.
///
/// Use [`unmarshal`](RawFrame::unmarshal) to decode the payload into a
/// concrete type, or inspect the fields directly for routing purposes.
///
/// # Lifetime
///
/// `'t` is tied to the input buffer passed to [`Messager::unpack`].
/// The payload slice borrows from that buffer with no copying.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawFrame<'t> {
    /// Command ID of the frame.
    pub(crate) cmd_id: u16,
    /// Sequence number of the frame.
    pub(crate) sequence: u8,
    /// Raw payload bytes.
    pub(crate) payload: &'t [u8],
}

impl<'t> RawFrame<'t> {
    /// Returns the command ID from the frame header.
    pub fn cmd_id(&self) -> u16 {
        self.cmd_id
    }

    /// Returns the sequence number from the frame header.
    pub fn sequence(&self) -> u8 {
        self.sequence
    }

    /// Decode the payload into a typed message.
    ///
    /// Verifies that the frame's command ID matches [`Marshaler::CMD_ID`] and that the
    /// payload length matches `M::PAYLOAD_SIZE`, then delegates to `M::unmarshal`.
    ///
    /// # Errors
    ///
    /// - [`MarshalerError::InvalidCmdID`] — the frame command ID does not match `M`.
    /// - [`MarshalerError::InvalidDataLength`] — payload length does not match
    ///   `M::PAYLOAD_SIZE`.
    /// - Any error returned by [`M::unmarshal`](crate::Marshaler::unmarshal).
    pub fn unmarshal<M: ImplUnMarshal>(&self) -> Result<M, MarshalerError> {
        if M::CMD_ID != self.cmd_id {
            return Err(MarshalerError::InvalidCmdID {
                expected: M::CMD_ID,
                found: self.cmd_id,
            });
        }

        if self.payload.len() != M::PAYLOAD_SIZE as usize {
            return Err(MarshalerError::InvalidDataLength {
                expected: M::PAYLOAD_SIZE as usize,
                found: self.payload.len(),
            });
        }

        M::unmarshal(self.payload)
    }

    /// Returns the raw payload bytes.
    ///
    /// The slice borrows directly from the input buffer passed to
    /// [`Messager::unpack`]. It contains only the payload data —
    /// no header, command ID, or CRC.
    pub fn payload(&self) -> &'t [u8] {
        self.payload
    }
}
