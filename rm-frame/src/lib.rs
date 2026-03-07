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
//! # Typical Usage
//!
//! 1. Implement [`Marshaler`] for your message types
//! 2. Create a [`Messager`] with a chosen [`Validator`]
//! 3. Use [`pack`](Messager::pack) to encode frames
//! 4. Use [`unpack`](Messager::unpack) to decode frames into [`RawFrame`]
//! 5. Use [`Marshaler`] to convert [`RawFrame`] payloads to typed messages
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

#![cfg_attr(not(test), no_std)]

pub use crc8_dji::calculate as calc_dji8;
pub use crc16_dji::calculate as calc_dji16;
pub use error::{MarshalerError, PackError, UnPackError};
pub use frame::{Messager, RawFrame};
pub use marshaler::{ImplCommandMsg, ImplMarshal, ImplUnMarshal, Marshaler};
pub use validator::{DjiValidator, Validator};

mod crc16_dji;
mod crc8_dji;
mod error;
mod frame;
mod marshaler;
mod validator;

#[cfg(test)]
mod tests;
