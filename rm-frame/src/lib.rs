//! A lightweight binary framing protocol library.
//!
//! This crate provides a minimal, allocation-free framing layer
//! designed for embedded and resource-constrained environments.
//! It focuses on deterministic binary encoding, checksum validation,
//! and zero-copy decoding.
//!
//! # Architecture Overview
//!
//! - **[`Validator`]**
//!   Defines CRC algorithms used to validate frames.
//!
//! - **[`DjiValidator`]**
//!   A concrete validator using DJI-compatible CRC8 and CRC16.
//!
//! - **[`Marshaler`]**
//!   Describes how a typed payload is serialized into bytes and
//!   deserialized from raw payload data.
//!
//! - **[`Messager`]**
//!   Implements frame packing and unpacking, combining framing,
//!   validation, and payload marshaling.
//!
//! - **[`RawFrame`]**
//!   A validated, zero-copy view of a decoded frame.
//!
//! - **[`RemoteControl`]** (optional)
//!   A helper for encoding and decoding remote control messages.
//!
//! # Typical Usage
//!
//! 1. Implement [`Marshaler`] for your message types
//! 2. Create a [`Messager`] with a chosen [`Validator`]
//! 3. Use [`pack`](Messager::pack) to encode frames
//! 4. Use [`unpack`](Messager::unpack) to decode frames into [`RawFrame`]
//! 5. Use [`Marshaler`] to convert [`RawFrame`] payloads to typed messages
//! 6. Optionally, use [`RemoteControl`] for handling remote control data
//!
//! # Example
//!
//! ```rust
//! # use rm_frame::{Marshaler, MarshalerError, Messager, DjiValidator};
//! # type Error = Box<dyn core::error::Error>;
//! // Define a custom message type and implement Marshaler for it
//! struct MyMessage {
//!    value: u32,
//! }
//!
//! impl Marshaler for MyMessage {
//!    const CMD_ID: u16 = 0x1234;
//!    const PAYLOAD_SIZE: u16 = 4;
//!
//!    // If called manually, the caller must ensure that the destination
//!    // buffer is large enough to hold the serialized payload.
//!    fn marshal(&self, buf: &mut [u8]) -> Result<usize, MarshalerError> {
//!        buf[..4].copy_from_slice(&self.value.to_le_bytes());
//!        Ok(4)
//!    }
//!
//!   fn unmarshal(buf: &[u8]) -> Result<Self, MarshalerError> {
//!        Ok(Self {
//!            value: u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]),
//!        })
//!   }
//! }
//!
//! # fn main() -> Result<(), Error> {
//! let mut buffer = [0u8; 64];
//! let mut msger = Messager::<DjiValidator>::new( /* seq: */ 0 );
//!
//! let msg = MyMessage { value: 38 };
//!
//! let packed_len = msger.pack(&msg, &mut buffer)?; // seq++
//! println!("Packed Frame Length: {}", packed_len);
//!
//! let (decoded_msg, consumed): (MyMessage, _) = msger.unmarshal(&buffer)?;
//! assert_eq!(consumed, packed_len);
//! assert_eq!(decoded_msg.value, 38);
//!
//! let msg = MyMessage { value: 42 };
//!
//! let packed_len = msger.pack(&msg, &mut buffer)?; // seq++
//! println!("Packed Frame Length: {}", packed_len);
//!
//! let (raw_frame, consumed) = msger.unpack(&buffer)?;
//! assert_eq!(consumed, packed_len);
//! assert_eq!(raw_frame.cmd_id(), MyMessage::CMD_ID);
//! assert_eq!(raw_frame.sequence(), 1);
//!
//! let decoded_msg = MyMessage::unmarshal(raw_frame.payload())?;
//! assert_eq!(decoded_msg.value, 42);
//! #    Ok(())
//! # }
//! ```
//!
//! > [`RemoteControl`]'s Example See [README.md][README] for details.
//!
//! ---
//!
//! # Frame Layout
//!
//! ```text
//! +--------+--------+--------+--------+--------+---------+--------+
//! |  SOF   |  LEN   |  SEQ   |  CRC8  | CMD_ID |  DATA   | CRC16  |
//! +--------+--------+--------+--------+--------+---------+--------+
//! | 1 byte | 2 byte | 1 byte | 1 byte | 2 byte | N bytes | 2 byte |
//! +--------+--------+--------+--------+--------+---------+--------+
//! ```
//!
//! # Protocol Source
//!
//! See RoboMaster Resources Hub for official documentation and protocol details:
//! [RMU Communication Protocol](https://bbs.robomaster.com/wiki/20204847/811363)
//!
//! # Features
//!
//! - `defmt`: Derives `defmt::Format` on error types and
//!     key enums for embedded structured logging.
//! - `remote`: Adds support for encoding and decoding remote control messages,
//!     including switch states and joystick positions.
//!
//! ---
//!
//! # License
//!
//! This crate is licensed under the MIT License or the Apache License (Version 2.0).
//! See `LICENSE-MIT` and `LICENSE-APACHE` files in the repository for details.
//!
//! [README]: https://github.com/Salfa-04/rm-proto
//!

#![cfg_attr(not(test), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::unescaped_backticks)]
#![warn(rustdoc::invalid_html_tags)]
#![warn(missing_docs)]

pub use crc8_dji::calculate as calc_dji8;
pub use crc16_dji::calculate as calc_dji16;
pub use error::{MarshalerError, PackError, UnPackError};
pub use frame::{Messager, RawFrame};
pub use marshaler::{ImplCommandMsg, ImplMarshal, ImplUnMarshal, Marshaler};
pub use validator::{DjiValidator, Validator};

#[cfg(feature = "remote")]
pub use remote::{RemoteControl, Switch};

mod crc16_dji;
mod crc8_dji;
mod error;
mod frame;
mod marshaler;
mod validator;

#[cfg(feature = "remote")]
mod remote;

#[cfg(test)]
mod tests;
