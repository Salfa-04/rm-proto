use crate::private::*;

const SOF: u16 = 0x_53_A9;
const RC_CHANNEL_MID: i16 = 1024;
const RC_FRAME_LENGTH: usize = 21;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Switch {
    C = 0,
    N = 1,
    S = 2,
}

/// Image Receiver to Controlled Robot
#[derive(Debug)]
pub struct RemoteControl {
    datagroup1: AtomicU64,
    datagroup2: AtomicU64,
    keyboard_v: AtomicU16,
}

impl RemoteControl {
    pub fn new() -> Self {
        Self {
            datagroup1: AtomicU64::new(0),
            datagroup2: AtomicU64::new(0),
            keyboard_v: AtomicU16::new(0),
        }
    }

    /// Creates a new zero-initialized `RemoteControl`.
    ///
    /// # Safety
    ///
    /// All-zero bytes must be a valid representation for all fields.
    /// Atomic types are valid when zero-initialized on all supported platforms.
    pub const unsafe fn const_new() -> Self {
        unsafe { core::mem::zeroed() }
    }

    pub fn update(&self, data: &[u8]) -> Result<usize, UnPackError> {
        if data.len() < RC_FRAME_LENGTH {
            return Err(UnPackError::UnexpectedEnd { read: data.len() });
        }

        let sof = u16::from_le_bytes([data[0], data[1]]);
        if sof != SOF {
            return Err(UnPackError::MissingHeader { skip: 1 });
        }

        let crc = u16::from_le_bytes([data[19], data[20]]);
        let calc_crc = calc_dji16(&data[..19]);
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

impl Default for RemoteControl {
    fn default() -> Self {
        Self::new()
    }
}

impl RemoteControl {
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

impl RemoteControl {
    pub fn right_horizontal(&self) -> i16 {
        let datagroup1 = self.get_datagroup1();
        let right_horizontal = (datagroup1 & 0x7FF) as i16;
        right_horizontal - RC_CHANNEL_MID
    }

    pub fn right_vertical(&self) -> i16 {
        let datagroup1 = self.get_datagroup1();
        let right_vertical = ((datagroup1 >> 11) & 0x7FF) as i16;
        right_vertical - RC_CHANNEL_MID
    }

    pub fn left_vertical(&self) -> i16 {
        let datagroup1 = self.get_datagroup1();
        let left_vertical = ((datagroup1 >> 22) & 0x7FF) as i16;
        left_vertical - RC_CHANNEL_MID
    }

    pub fn left_horizontal(&self) -> i16 {
        let datagroup1 = self.get_datagroup1();
        let left_horizontal = ((datagroup1 >> 33) & 0x7FF) as i16;
        left_horizontal - RC_CHANNEL_MID
    }

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

    pub fn pause(&self) -> bool {
        let datagroup1 = self.get_datagroup1();
        ((datagroup1 >> 46) & 0x1) != 0
    }

    pub fn left_fn(&self) -> bool {
        let datagroup1 = self.get_datagroup1();
        ((datagroup1 >> 47) & 0x1) != 0
    }

    pub fn right_fn(&self) -> bool {
        let datagroup1 = self.get_datagroup1();
        ((datagroup1 >> 48) & 0x1) != 0
    }

    pub fn wheel(&self) -> i16 {
        let datagroup1 = self.get_datagroup1();
        let wheel = ((datagroup1 >> 49) & 0x7FF) as i16;
        wheel - RC_CHANNEL_MID
    }

    pub fn trigger(&self) -> bool {
        let datagroup1 = self.get_datagroup1();
        ((datagroup1 >> 60) & 0x1) != 0
    }
}

impl RemoteControl {
    pub fn mouse_vx(&self) -> i16 {
        let datagroup2 = self.get_datagroup2();
        (datagroup2 & 0xFFFF) as i16
    }

    pub fn mouse_vy(&self) -> i16 {
        let datagroup2 = self.get_datagroup2();
        ((datagroup2 >> 16) & 0xFFFF) as i16
    }

    pub fn mouse_vz(&self) -> i16 {
        let datagroup2 = self.get_datagroup2();
        ((datagroup2 >> 32) & 0xFFFF) as i16
    }

    pub fn left_button(&self) -> bool {
        let datagroup2 = self.get_datagroup2();
        ((datagroup2 >> 48) & 0x3) != 0
    }

    pub fn right_button(&self) -> bool {
        let datagroup2 = self.get_datagroup2();
        ((datagroup2 >> 50) & 0x3) != 0
    }

    pub fn mid_button(&self) -> bool {
        let datagroup2 = self.get_datagroup2();
        ((datagroup2 >> 52) & 0x3) != 0
    }
}

impl RemoteControl {
    pub fn keyboard_w(&self) -> bool {
        self.get_keyboard_v() & (1 << 0) != 0
    }

    pub fn keyboard_s(&self) -> bool {
        (self.get_keyboard_v() & (1 << 1)) != 0
    }

    pub fn keyboard_a(&self) -> bool {
        (self.get_keyboard_v() & (1 << 2)) != 0
    }

    pub fn keyboard_d(&self) -> bool {
        (self.get_keyboard_v() & (1 << 3)) != 0
    }

    pub fn keyboard_shift(&self) -> bool {
        (self.get_keyboard_v() & (1 << 4)) != 0
    }

    pub fn keyboard_ctrl(&self) -> bool {
        (self.get_keyboard_v() & (1 << 5)) != 0
    }

    pub fn keyboard_q(&self) -> bool {
        (self.get_keyboard_v() & (1 << 6)) != 0
    }

    pub fn keyboard_e(&self) -> bool {
        (self.get_keyboard_v() & (1 << 7)) != 0
    }

    pub fn keyboard_r(&self) -> bool {
        (self.get_keyboard_v() & (1 << 8)) != 0
    }

    pub fn keyboard_f(&self) -> bool {
        (self.get_keyboard_v() & (1 << 9)) != 0
    }

    pub fn keyboard_g(&self) -> bool {
        (self.get_keyboard_v() & (1 << 10)) != 0
    }

    pub fn keyboard_z(&self) -> bool {
        (self.get_keyboard_v() & (1 << 11)) != 0
    }

    pub fn keyboard_x(&self) -> bool {
        (self.get_keyboard_v() & (1 << 12)) != 0
    }

    pub fn keyboard_c(&self) -> bool {
        (self.get_keyboard_v() & (1 << 13)) != 0
    }

    pub fn keyboard_v(&self) -> bool {
        (self.get_keyboard_v() & (1 << 14)) != 0
    }

    pub fn keyboard_b(&self) -> bool {
        (self.get_keyboard_v() & (1 << 15)) != 0
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for RemoteControl {
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
