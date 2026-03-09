//! Error types for the binframe crate.

use core::error::Error as StdError;
use core::fmt::{Display, Formatter, Result as FmtResult};

/// Errors returned by
///     [`Marshaler::marshal`](crate::Marshaler::marshal),
///     [`Marshaler::unmarshal`](crate::Marshaler::unmarshal),
/// and [`RawFrame::unmarshal`](crate::RawFrame::unmarshal).
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MarshalerError {
    /// The frame's command ID does not match the expected type.
    ///
    /// Returned by
    ///     [`RawFrame::unmarshal`](crate::RawFrame::unmarshal)
    /// when [`M::CMD_ID`](crate::Marshaler::CMD_ID) differs from the
    /// command ID in the decoded frame.
    InvalidCmdID {
        /// The expected command ID for the target type.
        expected: u16,
        /// The actual command ID found in the frame.
        found: u16,
    },

    /// The payload length does not match [`Marshaler::PAYLOAD_SIZE`](crate::Marshaler::PAYLOAD_SIZE).
    ///
    /// Returned by
    ///     [`RawFrame::unmarshal`](crate::RawFrame::unmarshal)
    /// when the payload size in the frame differs from the size the target type expects.
    InvalidDataLength {
        /// The expected payload size for the target type.
        expected: usize,
        /// The actual payload size found in the frame.
        found: usize,
    },

    /// The destination buffer is too small to hold the serialized payload.
    ///
    /// `need` is the minimum buffer size required, in bytes.
    BufferTooSmall {
        /// The minimum buffer size required, in bytes.
        need: usize,
    },

    /// An error specific to the payload type prevented successful marshaling or unmarshaling.
    ///
    /// This is a catch-all variant for errors that do not fit into the above categories.
    /// The inner error type and message are determined by the specific implementation of
    /// [`Marshaler::marshal`](crate::Marshaler::marshal) or
    /// [`Marshaler::unmarshal`](crate::Marshaler::unmarshal).
    Unexpected {
        /// The error code that user defined in their marshaler implementation.
        code: usize,
        /// The error message that user defined in their marshaler implementation.
        message: &'static str,
    },
}

impl From<()> for MarshalerError {
    fn from(_: ()) -> Self {
        Self::Unexpected {
            code: 0,
            message: "Unit Type Error",
        }
    }
}

impl From<usize> for MarshalerError {
    fn from(code: usize) -> Self {
        Self::Unexpected {
            code,
            message: "Unexpected Error",
        }
    }
}

impl From<(usize,)> for MarshalerError {
    fn from((code,): (usize,)) -> Self {
        Self::Unexpected {
            code,
            message: "Unexpected Error",
        }
    }
}

impl From<(&'static str, usize)> for MarshalerError {
    fn from((message, code): (&'static str, usize)) -> Self {
        Self::Unexpected { code, message }
    }
}

impl From<(usize, &'static str)> for MarshalerError {
    fn from((code, message): (usize, &'static str)) -> Self {
        Self::Unexpected { code, message }
    }
}

/// Errors returned by [`Messager::pack`](crate::Messager::pack).
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PackError {
    /// The destination buffer is too small to hold the complete frame.
    BufferTooSmall {
        /// The minimum required size in bytes:
        /// `5 (header) + 2 (CMD_ID) + PAYLOAD_SIZE + 2 (CRC16)`.
        need: usize,
    },

    /// The marshaler returned a byte count that does not equal
    /// [`Marshaler::PAYLOAD_SIZE`](crate::Marshaler::PAYLOAD_SIZE).
    ///
    /// This indicates a broken [`Marshaler`](crate::Marshaler) implementation.
    InvalidPayloadSize {
        /// The expected payload size for the target type.
        expected: usize,
        /// The actual payload size returned by the marshaler.
        found: usize,
    },

    /// The marshaler returned an error.
    MarshalerError(MarshalerError),
}

impl From<MarshalerError> for PackError {
    fn from(err: MarshalerError) -> Self {
        Self::MarshalerError(err)
    }
}

/// Errors returned by
///     [`Messager::unpack`](crate::Messager::unpack)
/// and [`Messager::unmarshal`](crate::Messager::unmarshal).
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UnPackError {
    /// The input does not start with the SOF byte (`0xA5`), but a SOF byte
    /// was found later in the buffer.
    ///
    /// `skip` is the byte offset of the first SOF byte found.
    /// Discard that many bytes from the input, then retry.
    ReSync {
        /// The byte offset of the first SOF byte found in the input buffer.
        skip: usize,
    },

    /// No SOF byte was found anywhere in the input buffer.
    ///
    /// `skip` equals the buffer length.
    /// Discard the entire buffer and wait for more data before retrying.
    MissingHeader {
        /// The length of the input buffer, which should be discarded.
        skip: usize,
    },

    /// The frame is truncated; more bytes are needed to complete it.
    ///
    /// `read` is the number of bytes currently available.
    /// Keep the existing bytes and wait for more data before retrying.
    UnexpectedEnd {
        /// The number of bytes currently available in the input buffer.
        read: usize,
    },

    /// CRC validation failed (header CRC8 or frame CRC16).
    ///
    /// `at` is the cursor position when the failure was detected.
    /// Call [`UnPackError::skip`] to determine how many bytes to discard.
    InvalidChecksum {
        /// The cursor position in the input buffer where the checksum failure was detected.
        at: usize,
    },

    /// The payload could not be decoded into the target type.
    ///
    /// Wraps a [`MarshalerError`] from [`RawFrame::unmarshal`](crate::RawFrame::unmarshal).
    MarshalerError(MarshalerError),
}

impl UnPackError {
    /// Returns the number of bytes to discard before retrying a parse.
    ///
    /// Use this when processing a continuous byte stream to advance the read
    /// position past invalid or incomplete data:
    ///
    /// | Variant | Returned value | Action |
    /// |---------|----------------|--------|
    /// | [`ReSync`](Self::ReSync) | offset of next SOF | discard bytes, retry |
    /// | [`MissingHeader`](Self::MissingHeader) | buffer length | discard all, wait for data |
    /// | [`UnexpectedEnd`](Self::UnexpectedEnd) | `0` | wait for more data at current position |
    /// | [`InvalidChecksum`](Self::InvalidChecksum) | cursor at failure | discard frame, retry |
    /// | [`MarshalerError`](Self::MarshalerError) | `0` | frame was consumed; handle decode error |
    pub fn skip(&self) -> usize {
        match self {
            Self::MissingHeader { skip } => *skip,
            Self::ReSync { skip } => *skip,
            Self::UnexpectedEnd { .. } => 0,
            Self::InvalidChecksum { at } => *at,
            Self::MarshalerError(_) => 0,
        }
    }
}

impl From<MarshalerError> for UnPackError {
    fn from(err: MarshalerError) -> Self {
        Self::MarshalerError(err)
    }
}

impl StdError for MarshalerError {}
impl Display for MarshalerError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::InvalidCmdID { expected, found } => write!(
                f,
                "Invalid Command ID: expected {}, found {}",
                expected, found
            ),
            Self::InvalidDataLength { expected, found } => {
                write!(
                    f,
                    "Invalid Data Length: expected {} bytes, found {} bytes",
                    expected, found
                )
            }
            Self::BufferTooSmall { need } => {
                write!(f, "Buffer too Small: need {} bytes", need)
            }
            Self::Unexpected { code, message } => {
                write!(
                    f,
                    "Unexpected Marshaler Error: code {}, message: {}",
                    code, message
                )
            }
        }
    }
}

impl Display for PackError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::BufferTooSmall { need } => {
                write!(f, "Buffer too Small: need {} bytes", need)
            }
            Self::InvalidPayloadSize { expected, found } => {
                write!(
                    f,
                    "Invalid Payload Size: expected {}, found {}",
                    expected, found
                )
            }
            Self::MarshalerError(e) => {
                write!(f, "Marshaler Error: {}", e)
            }
        }
    }
}
impl StdError for PackError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::MarshalerError(e) => Some(e),
            _ => None,
        }
    }
}

impl Display for UnPackError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::ReSync { skip } => write!(f, "ReSync Needed: skip {} bytes", skip),
            Self::MissingHeader { skip } => write!(f, "Missing Header: skip {} bytes", skip),
            Self::UnexpectedEnd { read } => write!(f, "Unexpected End: read {} bytes", read),
            Self::InvalidChecksum { at } => write!(f, "Invalid Checksum: at {} bytes", at),
            Self::MarshalerError(e) => write!(f, "Marshaler Error: {}", e),
        }
    }
}
impl StdError for UnPackError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::MarshalerError(e) => Some(e),
            _ => None,
        }
    }
}
