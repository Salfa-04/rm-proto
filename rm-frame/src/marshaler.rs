//! Defines the [`Marshaler`] trait and related traits for payload serialization and deserialization.

use crate::MarshalerError;

type Result<T> = core::result::Result<T, MarshalerError>;

///
/// Payload marshaling interface.
///
/// [`Marshaler`] defines how a message payload is:
/// - Serialized into raw bytes ([`Marshaler::marshal`])
/// - Deserialized from raw bytes ([`Marshaler::unmarshal`])
///
/// Each payload type corresponds to exactly one command ID.
///
#[doc(alias("Marshal", "Unmarshal", "Serialize", "Deserialize"))]
pub trait Marshaler: Sized {
    /// Command ID associated with this payload type.
    const CMD_ID: u16;
    /// Expected size of the payload in bytes.
    const PAYLOAD_SIZE: u16;

    ///
    /// Serialize the payload into the destination buffer.
    ///
    /// Returns the number of bytes written on success.
    ///
    /// # Notes
    ///
    /// If called manually, the caller must ensure
    /// that the destination buffer is large enough
    /// to hold the serialized payload.
    ///
    /// # Errors
    ///
    /// Returns an error if the destination buffer
    /// is too small or the payload cannot be encoded.
    ///
    fn marshal(&self, dst: &mut [u8]) -> Result<usize>;

    ///
    /// Deserialize a payload from raw bytes.
    ///
    /// The input slice contains only the payload portion
    /// (no header, command ID, or CRC).
    ///
    /// Implementations must not depend on framing details.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is invalid
    /// or does not match the expected payload format.
    ///
    fn unmarshal(raw: &[u8]) -> Result<Self>;
}

/// Provides the command ID and payload size constants for a message type.
///
/// This trait is a sub-trait of [`Marshaler`] and is automatically implemented
/// for any type that implements [`Marshaler`]. It can also be implemented
/// directly when only the metadata constants are needed.
pub trait ImplCommandMsg: Sized {
    /// See [`Marshaler::CMD_ID`] for details.
    const CMD_ID: u16;
    /// See [`Marshaler::PAYLOAD_SIZE`] for details.
    const PAYLOAD_SIZE: u16;
}

impl<T: Marshaler> ImplCommandMsg for T {
    const CMD_ID: u16 = T::CMD_ID;
    const PAYLOAD_SIZE: u16 = T::PAYLOAD_SIZE;
}

/// Payload serialization half of [`Marshaler`].
///
/// Automatically implemented for any type that implements [`Marshaler`].
/// Implement this trait directly — without implementing [`Marshaler`] —
/// to create an encode-only type that does not support deserialization.
///
/// Used as the bound on [`Messager::pack`](crate::Messager::pack).
pub trait ImplMarshal: ImplCommandMsg {
    /// See [`Marshaler::marshal`] for details.
    fn marshal(&self, dst: &mut [u8]) -> Result<usize>;
}

impl<T: Marshaler> ImplMarshal for T {
    fn marshal(&self, dst: &mut [u8]) -> Result<usize> {
        T::marshal(self, dst)
    }
}

/// Payload deserialization half of [`Marshaler`].
///
/// Automatically implemented for any type that implements [`Marshaler`].
/// Implement this trait directly — without implementing [`Marshaler`] —
/// to create a decode-only type that does not support serialization.
///
/// Used as the bound on [`Messager::unmarshal`](crate::Messager::unmarshal) and [`RawFrame::unmarshal`](crate::RawFrame::unmarshal).
pub trait ImplUnMarshal: ImplCommandMsg {
    /// See [`Marshaler::unmarshal`] for details.
    fn unmarshal(raw: &[u8]) -> Result<Self>;
}

impl<T: Marshaler> ImplUnMarshal for T {
    fn unmarshal(raw: &[u8]) -> Result<Self> {
        T::unmarshal(raw)
    }
}
