use crate::{DjiValidator, Marshaler, Messager};
use crate::{ImplCommandMsg, RemoteControl, Switch};
use crate::{MarshalerError, PackError, UnPackError};
use crate::{calc_dji8, calc_dji16};

#[test]
fn test_dji_crc8() {
    let data = b"123456789";
    assert_eq!(calc_dji8(data), 0x0B);
}

#[test]
fn test_dji_crc16() {
    let data = b"123456789";
    assert_eq!(calc_dji16(data), 0x6F91);
}

struct TestCase<const N: usize> {
    payload: [u8; N],
}

impl<const N: usize> Marshaler for TestCase<N> {
    const CMD_ID: u16 = 0x1234;
    const PAYLOAD_SIZE: u16 = N as u16;

    fn marshal(&self, dst: &mut [u8]) -> Result<usize, MarshalerError> {
        dst[..N].copy_from_slice(&self.payload);
        Ok(N)
    }

    fn unmarshal(src: &[u8]) -> Result<Self, MarshalerError> {
        let mut payload = [0u8; N];
        payload.copy_from_slice(&src[..N]);
        Ok(Self { payload })
    }
}

impl<const N: usize> TestCase<N> {
    fn new(payload: [u8; N]) -> Self {
        Self { payload }
    }
}

#[test]
fn test_encode_decode() {
    let mut msger: Messager<DjiValidator> = Messager::new(0);

    let test = TestCase::new([1, 2, 3, 4, 5]);
    let mut buffer = [0u8; 64];

    let size_a = msger.pack(&test, &mut buffer).unwrap();
    let (raw, size_b) = msger.unpack(&buffer[..size_a]).unwrap();

    let this = raw.unmarshal::<TestCase<5>>().unwrap();

    println!("Encoded size: {}", size_a);
    println!("Encoded data: {:X?}", &buffer[..size_a]);
    println!("Decoded size: {}", size_b);
    println!("Decoded payload: {:X?}", this.payload);

    assert_eq!(size_a, size_b);
    assert_eq!(raw.cmd_id(), <TestCase<5> as ImplCommandMsg>::CMD_ID);
    assert_eq!(test.payload, this.payload);
}

#[test]
fn test_invalid_decode() {
    let invalid_data = [0u8; 10];
    let msger: Messager<DjiValidator> = Messager::new(0);

    assert!(matches!(
        msger.unpack(&invalid_data),
        Err(UnPackError::MissingHeader { skip: 10 })
    ));
}

#[test]
fn test_validate_decode() {
    let valid_data = [
        0xA5, 0x5, 0x0, 0x56, 0xF0, // Header
        0x34, 0x12, // CMD ID
        0x1, 0x2, 0x3, 0x4, 0x5, // Data
        0x84, 0x71, // Tail CRC
    ];
    let msger: Messager<DjiValidator> = Messager::new(0);

    assert!(msger.unpack(&valid_data).is_ok());
}

#[test]
fn test_encode() {
    let test = TestCase::new([1, 2, 3, 4, 5]);
    let mut buffer = [0u8; 64];

    let mut msger: Messager<DjiValidator> = Messager::new(0x56);
    let size = msger.pack(&test, &mut buffer).unwrap();

    let expected: [u8; 14] = [
        0xA5, 0x5, 0x0, 0x56, 0xF0, // Header
        0x34, 0x12, // CMD ID
        0x1, 0x2, 0x3, 0x4, 0x5, // Data
        0x84, 0x71, // Tail CRC
    ];

    assert_eq!(&buffer[..size], &expected);
}

#[test]
fn test_insufficient_buffer() {
    let test = TestCase::new([1, 2, 3, 4, 5]);
    let mut buffer = [0u8; 10]; // Intentionally small buffer
    let mut msger: Messager<DjiValidator> = Messager::new(0x56);
    let result = msger.pack(&test, &mut buffer);
    assert!(matches!(
        result,
        Err(PackError::BufferTooSmall { need: 14 })
    ));
}

#[test]
fn test_sof_not_found() {
    let invalid_data = [
        0x5, 0x0, 0x56, 0xF0, // Header
        0x34, 0x12, // CMD ID
        0x1, 0x2, 0x3, 0x4, 0x5, // Data
        0x84, 0x71, // Tail CRC
    ];
    let msger: Messager<DjiValidator> = Messager::new(0x56);
    let result = msger.unpack(&invalid_data);
    assert!(matches!(
        result,
        Err(UnPackError::MissingHeader { skip: 13 })
    ));
}

#[test]
fn test_invalid_header_checksum() {
    let invalid_data = [
        0xA5, 0x5, 0xFF, 0x56, 0xF0, // Invalid Header
        0x34, 0x12, // CMD ID
        0x1, 0x2, 0x3, 0x4, 0x5, // Data
        0x84, 0x71, // Tail CRC
    ];
    let msger: Messager<DjiValidator> = Messager::new(0x56);
    let result = msger.unpack(&invalid_data);
    assert!(matches!(
        result,
        Err(UnPackError::InvalidChecksum { at: 5 })
    ));
}

#[test]
fn test_invalid_tail_checksum() {
    let invalid_data = [
        0xA5, 0x5, 0x0, 0x56, 0xF0, // Header
        0x34, 0x12, // CMD ID
        0x1, 0x2, 0x3, 0x4, 0x5, // Data
        0x00, 0x00, // Invalid Tail CRC
    ];
    let msger: Messager<DjiValidator> = Messager::new(0x56);
    let result = msger.unpack(&invalid_data);
    assert!(matches!(
        result,
        Err(UnPackError::InvalidChecksum { at: 14 })
    ));
}

#[test]
fn test_resync() {
    // Two garbage bytes precede a valid frame; unpack should report ReSync { skip: 2 }.
    let data = [
        0x00, 0x01, // garbage prefix
        0xA5, 0x5, 0x0, 0x56, 0xF0, // valid SOF at index 2
        0x34, 0x12, 0x1, 0x2, 0x3, 0x4, 0x5, 0x84, 0x71,
    ];
    let msger: Messager<DjiValidator> = Messager::new(0x56);
    assert!(matches!(
        msger.unpack(&data),
        Err(UnPackError::ReSync { skip: 2 })
    ));
}

#[test]
fn test_unexpected_end_truncated_header() {
    // Only 3 bytes — not enough to read the 5-byte header.
    let data = [0xA5, 0x05, 0x00];
    let msger: Messager<DjiValidator> = Messager::new(0);
    assert!(matches!(
        msger.unpack(&data),
        Err(UnPackError::UnexpectedEnd { read: 3 })
    ));
}

#[test]
fn test_unexpected_end_truncated_cmd_id() {
    // Valid header, but only one byte of the two-byte CMD_ID is present.
    let data = [
        0xA5, 0x5, 0x0, 0x56, 0xF0, // valid header
        0x34, // only first byte of CMD_ID
    ];
    let msger: Messager<DjiValidator> = Messager::new(0x56);
    assert!(matches!(
        msger.unpack(&data),
        Err(UnPackError::UnexpectedEnd { read: 6 })
    ));
}

#[test]
fn test_unexpected_end_truncated_payload() {
    // Valid header + complete CMD_ID, but only 2 of the 5 declared payload bytes.
    let data = [
        0xA5, 0x5, 0x0, 0x56, 0xF0, // valid header (LEN = 5)
        0x34, 0x12, // CMD_ID
        0x1, 0x2, // only 2 of 5 payload bytes
    ];
    let msger: Messager<DjiValidator> = Messager::new(0x56);
    assert!(matches!(
        msger.unpack(&data),
        Err(UnPackError::UnexpectedEnd { read: 9 })
    ));
}

#[test]
fn test_unexpected_end_missing_tail() {
    // Full header + CMD_ID + payload, but tail CRC is absent.
    let data = [
        0xA5, 0x5, 0x0, 0x56, 0xF0, // valid header
        0x34, 0x12, // CMD_ID
        0x1, 0x2, 0x3, 0x4, 0x5, // payload
             // tail CRC missing
    ];
    let msger: Messager<DjiValidator> = Messager::new(0x56);
    assert!(matches!(
        msger.unpack(&data),
        Err(UnPackError::UnexpectedEnd { read: 12 })
    ));
}

#[test]
fn test_sequence_number() {
    let mut msger: Messager<DjiValidator> = Messager::new(0x42);
    let test = TestCase::new([1, 2, 3, 4, 5]);
    let mut buf1 = [0u8; 64];
    let mut buf2 = [0u8; 64];

    let size1 = msger.pack(&test, &mut buf1).unwrap();
    let size2 = msger.pack(&test, &mut buf2).unwrap();

    let (raw1, _) = msger.unpack(&buf1[..size1]).unwrap();
    let (raw2, _) = msger.unpack(&buf2[..size2]).unwrap();

    assert_eq!(raw1.sequence(), 0x42);
    assert_eq!(raw2.sequence(), 0x43);
}

#[test]
fn test_sequence_wrapping() {
    let mut msger: Messager<DjiValidator> = Messager::new(0xFF);
    let test = TestCase::new([1, 2, 3, 4, 5]);
    let mut buf1 = [0u8; 64];
    let mut buf2 = [0u8; 64];

    let size1 = msger.pack(&test, &mut buf1).unwrap();
    let size2 = msger.pack(&test, &mut buf2).unwrap();

    let (raw1, _) = msger.unpack(&buf1[..size1]).unwrap();
    let (raw2, _) = msger.unpack(&buf2[..size2]).unwrap();

    assert_eq!(raw1.sequence(), 0xFF);
    assert_eq!(raw2.sequence(), 0x00); // wraps
}

#[test]
fn test_raw_frame_accessors() {
    let mut msger: Messager<DjiValidator> = Messager::new(0x7E);
    let test = TestCase::new([0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);
    let mut buffer = [0u8; 64];

    let size = msger.pack(&test, &mut buffer).unwrap();
    let (raw, _) = msger.unpack(&buffer[..size]).unwrap();

    assert_eq!(raw.cmd_id(), <TestCase<5> as ImplCommandMsg>::CMD_ID);
    assert_eq!(raw.sequence(), 0x7E);
    assert_eq!(raw.payload(), &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);
}

#[test]
fn test_zero_payload() {
    let mut msger: Messager<DjiValidator> = Messager::new(0);
    let test = TestCase::new([]);
    let mut buffer = [0u8; 64];

    // 5 (header) + 2 (cmd_id) + 0 (payload) + 2 (crc16) = 9 bytes
    let size = msger.pack(&test, &mut buffer).unwrap();
    assert_eq!(size, 9);

    let (raw, consumed) = msger.unpack(&buffer[..size]).unwrap();
    assert_eq!(consumed, 9);
    assert_eq!(raw.payload().len(), 0);
}

#[test]
fn test_unmarshal_direct() {
    let mut msger: Messager<DjiValidator> = Messager::new(0);
    let test = TestCase::new([10, 20, 30, 40, 50]);
    let mut buffer = [0u8; 64];

    let size = msger.pack(&test, &mut buffer).unwrap();
    let (decoded, consumed): (TestCase<5>, usize) = msger.unmarshal(&buffer[..size]).unwrap();

    assert_eq!(decoded.payload, test.payload);
    assert_eq!(consumed, size);
}

struct AltCase {
    _pad: [u8; 5],
}

impl Marshaler for AltCase {
    const CMD_ID: u16 = 0x5678; // differs from TestCase::CMD_ID
    const PAYLOAD_SIZE: u16 = 5;

    fn marshal(&self, dst: &mut [u8]) -> Result<usize, MarshalerError> {
        dst[..5].copy_from_slice(&self._pad);
        Ok(5)
    }

    fn unmarshal(src: &[u8]) -> Result<Self, MarshalerError> {
        let mut pad = [0u8; 5];
        pad.copy_from_slice(&src[..5]);
        Ok(Self { _pad: pad })
    }
}

#[test]
fn test_wrong_cmd_id() {
    let mut msger: Messager<DjiValidator> = Messager::new(0);
    let test = TestCase::new([1, 2, 3, 4, 5]); // CMD_ID = 0x1234
    let mut buffer = [0u8; 64];

    let size = msger.pack(&test, &mut buffer).unwrap();
    let (raw, _) = msger.unpack(&buffer[..size]).unwrap();

    // Attempt to decode as AltCase (CMD_ID = 0x5678) must fail.
    assert!(matches!(
        raw.unmarshal::<AltCase>(),
        Err(MarshalerError::InvalidCmdID {
            expected: 0x5678,
            found: 0x1234
        })
    ));
}

#[test]
fn test_wrong_payload_size() {
    let mut msger: Messager<DjiValidator> = Messager::new(0);
    let test = TestCase::new([1, 2, 3, 4, 5]); // PAYLOAD_SIZE = 5
    let mut buffer = [0u8; 64];

    let size = msger.pack(&test, &mut buffer).unwrap();
    let (raw, _) = msger.unpack(&buffer[..size]).unwrap();

    // TestCase<3> has the same CMD_ID (0x1234) but expects only 3 bytes.
    assert!(matches!(
        raw.unmarshal::<TestCase<3>>(),
        Err(MarshalerError::InvalidDataLength {
            expected: 3,
            found: 5
        })
    ));
}

struct BrokenMarshaler;

impl Marshaler for BrokenMarshaler {
    const CMD_ID: u16 = 0xDEAD;
    const PAYLOAD_SIZE: u16 = 5;

    fn marshal(&self, _dst: &mut [u8]) -> Result<usize, MarshalerError> {
        Ok(3) // intentionally wrong: claims 3 instead of PAYLOAD_SIZE (5)
    }

    fn unmarshal(_src: &[u8]) -> Result<Self, MarshalerError> {
        Ok(Self)
    }
}

#[test]
fn test_invalid_payload_size_error() {
    let mut msger: Messager<DjiValidator> = Messager::new(0);
    let mut buffer = [0u8; 64];
    assert!(matches!(
        msger.pack(&BrokenMarshaler, &mut buffer),
        Err(PackError::InvalidPayloadSize {
            expected: 5,
            found: 3
        })
    ));
}

#[test]
fn test_skip_values() {
    assert_eq!(UnPackError::ReSync { skip: 5 }.skip(), 5);
    assert_eq!(UnPackError::MissingHeader { skip: 10 }.skip(), 10);
    assert_eq!(UnPackError::UnexpectedEnd { read: 3 }.skip(), 0);
    assert_eq!(UnPackError::InvalidChecksum { at: 14 }.skip(), 14);
    assert_eq!(
        UnPackError::MarshalerError(MarshalerError::from(0usize)).skip(),
        0
    );
}

fn build_vt13_frame(
    ch0: u16,
    ch1: u16,
    ch2: u16,
    ch3: u16,
    switch: u8,
    pause: bool,
    left_fn: bool,
    right_fn: bool,
    wheel: u16,
    trigger: bool,
    mouse_vx: i16,
    mouse_vy: i16,
    mouse_vz: i16,
    left_button_bits: u8,
    right_button_bits: u8,
    mid_button_bits: u8,
    keyboard_mask: u16,
) -> [u8; 21] {
    const SOF: u16 = 0x53A9;
    const FRAME_LEN: usize = 21;

    let mut frame = [0u8; FRAME_LEN];

    let mut datagroup1 = 0u64;
    datagroup1 |= (ch0 as u64) & 0x7FF;
    datagroup1 |= ((ch1 as u64) & 0x7FF) << 11;
    datagroup1 |= ((ch2 as u64) & 0x7FF) << 22;
    datagroup1 |= ((ch3 as u64) & 0x7FF) << 33;
    datagroup1 |= ((switch as u64) & 0x3) << 44;
    datagroup1 |= (pause as u64) << 46;
    datagroup1 |= (left_fn as u64) << 47;
    datagroup1 |= (right_fn as u64) << 48;
    datagroup1 |= ((wheel as u64) & 0x7FF) << 49;
    datagroup1 |= (trigger as u64) << 60;

    let mut datagroup2 = 0u64;
    datagroup2 |= (mouse_vx as u16 as u64) & 0xFFFF;
    datagroup2 |= ((mouse_vy as u16 as u64) & 0xFFFF) << 16;
    datagroup2 |= ((mouse_vz as u16 as u64) & 0xFFFF) << 32;
    datagroup2 |= ((left_button_bits as u64) & 0x3) << 48;
    datagroup2 |= ((right_button_bits as u64) & 0x3) << 50;
    datagroup2 |= ((mid_button_bits as u64) & 0x3) << 52;

    frame[0..2].copy_from_slice(&SOF.to_le_bytes());
    frame[2..10].copy_from_slice(&datagroup1.to_le_bytes());
    frame[10..17].copy_from_slice(&datagroup2.to_le_bytes()[..7]);
    frame[17..19].copy_from_slice(&keyboard_mask.to_le_bytes());

    let crc = calc_dji16(&frame[..19]);
    frame[19..21].copy_from_slice(&crc.to_le_bytes());

    frame
}

#[test]
fn test_remote_update_decodes_full_payload() {
    const MID: i16 = 1024;

    let rc: RemoteControl<DjiValidator> = RemoteControl::new();
    let frame = build_vt13_frame(
        (MID + 100) as u16,
        (MID - 200) as u16,
        (MID + 300) as u16,
        (MID - 400) as u16,
        Switch::N as u8,
        true,
        false,
        true,
        (MID + 123) as u16,
        true,
        -1234,
        2345,
        -32768,
        0b01,
        0b10,
        0b11,
        (1 << 0) | (1 << 5) | (1 << 15),
    );

    let consumed = rc.update(&frame).unwrap();
    assert_eq!(consumed, 21);

    assert_eq!(rc.right_horizontal(), 100);
    assert_eq!(rc.right_vertical(), -200);
    assert_eq!(rc.left_vertical(), 300);
    assert_eq!(rc.left_horizontal(), -400);
    assert_eq!(rc.switch(), Switch::N);
    assert!(rc.pause());
    assert!(!rc.left_fn());
    assert!(rc.right_fn());
    assert_eq!(rc.wheel(), 123);
    assert!(rc.trigger());

    assert_eq!(rc.mouse_vx(), -1234);
    assert_eq!(rc.mouse_vy(), 2345);
    assert_eq!(rc.mouse_vz(), -32768);
    assert!(rc.left_button());
    assert!(rc.right_button());
    assert!(rc.mid_button());

    assert!(rc.keyboard_w());
    assert!(rc.keyboard_ctrl());
    assert!(rc.keyboard_b());
    assert!(!rc.keyboard_shift());
    assert!(!rc.keyboard_a());
}

#[test]
fn test_remote_update_rejects_short_frame() {
    let rc: RemoteControl<DjiValidator> = RemoteControl::new();
    let short = [0u8; 20];

    assert!(matches!(
        rc.update(&short),
        Err(UnPackError::UnexpectedEnd { read: 20 })
    ));
}

#[test]
fn test_remote_update_rejects_bad_sof() {
    const MID: i16 = 1024;

    let rc: RemoteControl<DjiValidator> = RemoteControl::new();
    let mut frame = build_vt13_frame(
        MID as u16,
        MID as u16,
        MID as u16,
        MID as u16,
        Switch::C as u8,
        false,
        false,
        false,
        MID as u16,
        false,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    );

    frame[0] ^= 0xFF;
    assert!(matches!(
        rc.update(&frame),
        Err(UnPackError::MissingHeader { skip: 1 })
    ));
}

#[test]
fn test_remote_update_rejects_bad_crc() {
    const MID: i16 = 1024;

    let rc: RemoteControl<DjiValidator> = RemoteControl::new();
    let mut frame = build_vt13_frame(
        MID as u16,
        MID as u16,
        MID as u16,
        MID as u16,
        Switch::S as u8,
        false,
        false,
        false,
        MID as u16,
        false,
        10,
        20,
        30,
        0,
        0,
        0,
        0,
    );

    frame[20] ^= 0x01;
    assert!(matches!(
        rc.update(&frame),
        Err(UnPackError::InvalidChecksum { at: 19 })
    ));
}
