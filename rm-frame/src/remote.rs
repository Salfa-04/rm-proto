//! VT03/VT13 uplink control frame decoder.
//!
//! This module decodes the 21-byte uplink control payload transmitted by
//! VT03/VT13 video transmitter links.
//!
//! The payload contains joystick channels, switch states, function keys,
//! mouse deltas/buttons, and keyboard bitmask data.
//!
//! This format is independent from the standard `Messager` frame format
//! (`SOF = 0xA5`) used by referee and custom command messages.
//!
//! # RC Frame Layout (21 bytes)
//!
//! All multi-byte values are little-endian.
//!
//! ```text
//! +---------+----------------------+----------------------+-----------+--------+
//! | Bytes   | Field                | Size                 | Note      | Source |
//! +---------+----------------------+----------------------+-----------+--------+
//! | 0..2    | SOF                  | 2 B                  | 0xA953    | raw    |
//! | 2..10   | Data Group 1         | 8 B                  | bitfield  | raw    |
//! | 10..17  | Data Group 2         | 7 B                  | bitfield  | raw    |
//! | 17..19  | Keyboard bitmask     | 2 B                  | 16 keys   | raw    |
//! | 19..21  | CRC16                | 2 B                  | over 0..19| raw    |
//! +---------+----------------------+----------------------+-----------+--------+
//! ```
//!
//! Internal storage expands Data Group 2 to `u64` by appending a zero byte as
//! the highest byte, so bit extraction stays simple.
//!
//! # Data Group 1 Bit Mapping (`u64`)
//!
//! ```text
//! bits  0..=10  : right_horizontal (11-bit channel, centered at 1024)
//! bits 11..=21  : right_vertical   (11-bit channel, centered at 1024)
//! bits 22..=32  : left_vertical    (11-bit channel, centered at 1024)
//! bits 33..=43  : left_horizontal  (11-bit channel, centered at 1024)
//! bits 44..=45  : switch           (0 = C, 1 = N, 2 = S)
//! bit       46  : pause            (Boolean, true if pressed)
//! bit       47  : left_fn          (Boolean, true if pressed)
//! bit       48  : right_fn         (Boolean, true if pressed)
//! bits 49..=59  : wheel            (11-bit channel, centered at 1024)
//! bit       60  : trigger          (Boolean, true if pressed)
//! ```
//!
//! # Data Group 2 Bit Mapping (`u64`)
//!
//! ```text
//! bits  0..=15  : mouse_vx (i16)
//! bits 16..=31  : mouse_vy (i16)
//! bits 32..=47  : mouse_vz (i16)
//! bits 48..=49  : left_button
//! bits 50..=51  : right_button
//! bits 52..=53  : mid_button
//! ```
//!
//! Button extraction treats a non-zero 2-bit value as `pressed` for
//! compatibility with existing upstream encoders.
//!

use crate::{DjiValidator, UnPackError, Validator};
use atomic::{AtomicU16, AtomicU64, Ordering::Relaxed};
use core::fmt::{Display, Formatter, Result as FmtResult};
use core::marker::PhantomData;

/// Start-of-frame marker for VT03/VT13 uplink stream
/// (`0xA953`, little-endian bytes `[0xA9, 0x53]`).
const SOF: u16 = 0x_53_A9;
/// Electrical neutral position for 11-bit analog channels.
const RC_CHANNEL_MID: i16 = 1024;
/// Fixed size of one remote-control frame in bytes.
const RC_FRAME_LENGTH: usize = 21;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[doc(alias("SwitchState", "3PosSwitch", "3WaySwitch"))]
pub enum Switch {
    /// Left position.
    C = 0,
    /// Middle position.
    N = 1,
    /// Right position.
    S = 2,
}

/// Decoded view of the remote-control state.
///
/// The struct is designed for lock-free sharing across tasks/threads in
/// embedded runtimes. `update` writes a new snapshot atomically per field, and
/// all getters are wait-free atomic loads.
///
/// # Generic Parameter
///
/// `V` is the validator type used for CRC16 checksum verification. It must
/// implement the [`Validator`] trait, which defines how to calculate the CRC16
/// checksum for the frame. By default, `V` is set to [`DjiValidator`],
/// which uses the standard DJI CRC16 algorithm.
#[derive(Debug)]
#[doc(alias("RemoteControlState", "RCState", "RemoteInput"))]
pub struct RemoteControl<V: Validator = DjiValidator> {
    /// Packed channels/switch/functions/wheel/trigger.
    datagroup1: AtomicU64,
    /// Packed mouse deltas and mouse buttons.
    datagroup2: AtomicU64,
    /// Packed keyboard bitmask (`W S A D Shift Ctrl Q E R F G Z X C V B`).
    keyboard_v: AtomicU16,
    /// Marker for the validator type.
    _marker: PhantomData<V>,
}

impl<V: Validator> RemoteControl<V> {
    /// Creates a zero-initialized remote-control state.
    pub const fn new() -> RemoteControl<V> {
        Self {
            datagroup1: AtomicU64::new(0),
            datagroup2: AtomicU64::new(0),
            keyboard_v: AtomicU16::new(0),
            _marker: PhantomData,
        }
    }

    /// Parses one 21-byte VT03/VT13 uplink frame and updates internal state.
    ///
    /// Returns the number of consumed bytes (`21`) on success.
    ///
    /// # Errors
    ///
    /// - [`UnPackError::UnexpectedEnd`]: input shorter than 21 bytes.
    /// - [`UnPackError::MissingHeader`]: SOF mismatch at frame start.
    /// - [`UnPackError::InvalidChecksum`]: CRC16 mismatch (tail bytes 19..21).
    pub fn update(&self, data: &[u8]) -> Result<usize, UnPackError> {
        if data.len() < RC_FRAME_LENGTH {
            return Err(UnPackError::UnexpectedEnd { read: data.len() });
        }

        let sof = u16::from_le_bytes([data[0], data[1]]);
        if sof != SOF {
            return Err(UnPackError::MissingHeader { skip: 1 });
        }

        let crc = u16::from_le_bytes([data[19], data[20]]);
        let calc_crc = V::calculate_crc16(&data[..19]);
        if crc != calc_crc {
            return Err(UnPackError::InvalidChecksum { at: 19 });
        }

        let data = &data[2..19];
        let datagroup1 = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        let datagroup2 = u64::from_le_bytes([
            data[8], data[9], data[10], data[11], data[12], data[13], data[14], 0,
        ]);
        let keyboard_v = u16::from_le_bytes([data[15], data[16]]);

        self.datagroup1.store(datagroup1, Relaxed);
        self.datagroup2.store(datagroup2, Relaxed);
        self.keyboard_v.store(keyboard_v, Relaxed);

        Ok(RC_FRAME_LENGTH)
    }
}

impl<V: Validator> RemoteControl<V> {
    fn get_datagroup1(&self) -> u64 {
        self.datagroup1.load(Relaxed)
    }

    fn get_datagroup2(&self) -> u64 {
        self.datagroup2.load(Relaxed)
    }

    fn get_keyboard_v(&self) -> u16 {
        self.keyboard_v.load(Relaxed)
    }
}

impl<V: Validator> RemoteControl<V> {
    /// Right stick horizontal channel (`ch0`), (±660, centered at 0).
    pub fn right_horizontal(&self) -> i16 {
        let datagroup1 = self.get_datagroup1();
        let right_horizontal = (datagroup1 & 0x7FF) as i16;
        right_horizontal - RC_CHANNEL_MID
    }

    /// Right stick vertical channel (`ch1`), (±660, centered at 0).
    pub fn right_vertical(&self) -> i16 {
        let datagroup1 = self.get_datagroup1();
        let right_vertical = ((datagroup1 >> 11) & 0x7FF) as i16;
        right_vertical - RC_CHANNEL_MID
    }

    /// Left stick vertical channel (`ch2`), (±660, centered at 0).
    pub fn left_vertical(&self) -> i16 {
        let datagroup1 = self.get_datagroup1();
        let left_vertical = ((datagroup1 >> 22) & 0x7FF) as i16;
        left_vertical - RC_CHANNEL_MID
    }

    /// Left stick horizontal channel (`ch3`), (±660, centered at 0).
    pub fn left_horizontal(&self) -> i16 {
        let datagroup1 = self.get_datagroup1();
        let left_horizontal = ((datagroup1 >> 33) & 0x7FF) as i16;
        left_horizontal - RC_CHANNEL_MID
    }

    /// Three-position switch state (`C`/`N`/`S`).
    pub fn switch(&self) -> Switch {
        let datagroup1 = self.get_datagroup1();
        let switch = ((datagroup1 >> 44) & 0x3) as u8;
        match switch {
            0 => Switch::C,
            1 => Switch::N,
            2 => Switch::S,
            // Note: Only 2Bits are Used for Switch
            _ => unreachable!(),
        }
    }

    /// Pause key state, true if pressed.
    pub fn pause(&self) -> bool {
        let datagroup1 = self.get_datagroup1();
        ((datagroup1 >> 46) & 0x1) != 0
    }

    /// Left function key state, true if pressed.
    pub fn left_fn(&self) -> bool {
        let datagroup1 = self.get_datagroup1();
        ((datagroup1 >> 47) & 0x1) != 0
    }

    /// Right function key state, true if pressed.
    pub fn right_fn(&self) -> bool {
        let datagroup1 = self.get_datagroup1();
        ((datagroup1 >> 48) & 0x1) != 0
    }

    /// Wheel channel, (±660, centered at 0).
    pub fn wheel(&self) -> i16 {
        let datagroup1 = self.get_datagroup1();
        let wheel = ((datagroup1 >> 49) & 0x7FF) as i16;
        wheel - RC_CHANNEL_MID
    }

    /// Trigger state, true if pressed.
    pub fn trigger(&self) -> bool {
        let datagroup1 = self.get_datagroup1();
        ((datagroup1 >> 60) & 0x1) != 0
    }
}

impl<V: Validator> RemoteControl<V> {
    /// Mouse X delta: [`i16`].
    pub fn mouse_vx(&self) -> i16 {
        let datagroup2 = self.get_datagroup2();
        (datagroup2 & 0xFFFF) as i16
    }

    /// Mouse Y delta: [`i16`].
    pub fn mouse_vy(&self) -> i16 {
        let datagroup2 = self.get_datagroup2();
        ((datagroup2 >> 16) & 0xFFFF) as i16
    }

    /// Mouse wheel/scroll delta: [`i16`].
    pub fn mouse_vz(&self) -> i16 {
        let datagroup2 = self.get_datagroup2();
        ((datagroup2 >> 32) & 0xFFFF) as i16
    }

    /// Left mouse button state, true if pressed.
    pub fn left_button(&self) -> bool {
        let datagroup2 = self.get_datagroup2();
        ((datagroup2 >> 48) & 0x3) != 0
    }

    /// Right mouse button state, true if pressed.
    pub fn right_button(&self) -> bool {
        let datagroup2 = self.get_datagroup2();
        ((datagroup2 >> 50) & 0x3) != 0
    }

    /// Middle mouse button state, true if pressed.
    pub fn mid_button(&self) -> bool {
        let datagroup2 = self.get_datagroup2();
        ((datagroup2 >> 52) & 0x3) != 0
    }
}

impl<V: Validator> RemoteControl<V> {
    /// Keyboard `W` key state, true if pressed.
    pub fn keyboard_w(&self) -> bool {
        self.get_keyboard_v() & (1 << 0) != 0
    }

    /// Keyboard `S` key state, true if pressed.
    pub fn keyboard_s(&self) -> bool {
        (self.get_keyboard_v() & (1 << 1)) != 0
    }

    /// Keyboard `A` key state, true if pressed.
    pub fn keyboard_a(&self) -> bool {
        (self.get_keyboard_v() & (1 << 2)) != 0
    }

    /// Keyboard `D` key state, true if pressed.
    pub fn keyboard_d(&self) -> bool {
        (self.get_keyboard_v() & (1 << 3)) != 0
    }

    /// Keyboard `Shift` key state, true if pressed.
    pub fn keyboard_shift(&self) -> bool {
        (self.get_keyboard_v() & (1 << 4)) != 0
    }

    /// Keyboard `Ctrl` key state, true if pressed.
    pub fn keyboard_ctrl(&self) -> bool {
        (self.get_keyboard_v() & (1 << 5)) != 0
    }

    /// Keyboard `Q` key state, true if pressed.
    pub fn keyboard_q(&self) -> bool {
        (self.get_keyboard_v() & (1 << 6)) != 0
    }

    /// Keyboard `E` key state, true if pressed.
    pub fn keyboard_e(&self) -> bool {
        (self.get_keyboard_v() & (1 << 7)) != 0
    }

    /// Keyboard `R` key state, true if pressed.
    pub fn keyboard_r(&self) -> bool {
        (self.get_keyboard_v() & (1 << 8)) != 0
    }

    /// Keyboard `F` key state, true if pressed.
    pub fn keyboard_f(&self) -> bool {
        (self.get_keyboard_v() & (1 << 9)) != 0
    }

    /// Keyboard `G` key state, true if pressed.
    pub fn keyboard_g(&self) -> bool {
        (self.get_keyboard_v() & (1 << 10)) != 0
    }

    /// Keyboard `Z` key state, true if pressed.
    pub fn keyboard_z(&self) -> bool {
        (self.get_keyboard_v() & (1 << 11)) != 0
    }

    /// Keyboard `X` key state, true if pressed.
    pub fn keyboard_x(&self) -> bool {
        (self.get_keyboard_v() & (1 << 12)) != 0
    }

    /// Keyboard `C` key state, true if pressed.
    pub fn keyboard_c(&self) -> bool {
        (self.get_keyboard_v() & (1 << 13)) != 0
    }

    /// Keyboard `V` key state, true if pressed.
    pub fn keyboard_v(&self) -> bool {
        (self.get_keyboard_v() & (1 << 14)) != 0
    }

    /// Keyboard `B` key state, true if pressed.
    pub fn keyboard_b(&self) -> bool {
        (self.get_keyboard_v() & (1 << 15)) != 0
    }
}

impl<V: Validator> Clone for RemoteControl<V> {
    fn clone(&self) -> Self {
        Self {
            datagroup1: AtomicU64::new(self.get_datagroup1()),
            datagroup2: AtomicU64::new(self.get_datagroup2()),
            keyboard_v: AtomicU16::new(self.get_keyboard_v()),
            _marker: PhantomData,
        }
    }
}

impl<V: Validator> Default for RemoteControl<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: Validator> Display for RemoteControl<V> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "{{\n\tRH: {}, RV: {}, LV: {}, LH: {},\n\tSW: {:?}, P(HII): {},\n\tFn L: {}, R: {},\n\twheel: {}, trigger: {},\n\tMouse: X: {}, Y: {}, Z: {},\n\tMouse: L: {}, M: {}, R: {},\n\tKeyBoard: 0b{:016b}\n}}",
            self.right_horizontal(),
            self.right_vertical(),
            self.left_vertical(),
            self.left_horizontal(),
            self.switch(),
            self.pause(),
            self.left_fn(),
            self.right_fn(),
            self.wheel(),
            self.trigger(),
            self.mouse_vx(),
            self.mouse_vy(),
            self.mouse_vz(),
            self.left_button(),
            self.mid_button(),
            self.right_button(),
            self.get_keyboard_v()
        )
    }
}

#[cfg(feature = "defmt")]
impl<V: Validator> defmt::Format for RemoteControl<V> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "{{\n\tRH: {}, RV: {}, LV: {}, LH: {},\n\tSW: {:?}, P(HII): {},\n\tFn L: {}, R: {},\n\twheel: {}, trigger: {},\n\tMouse: X: {}, Y: {}, Z: {},\n\tMouse: L: {}, M: {}, R: {},\n\tKeyBoard: 0b{:016b}\n}}",
            self.right_horizontal(),
            self.right_vertical(),
            self.left_vertical(),
            self.left_horizontal(),
            self.switch(),
            self.pause(),
            self.left_fn(),
            self.right_fn(),
            self.wheel(),
            self.trigger(),
            self.mouse_vx(),
            self.mouse_vy(),
            self.mouse_vz(),
            self.left_button(),
            self.mid_button(),
            self.right_button(),
            self.get_keyboard_v()
        );
    }
}
