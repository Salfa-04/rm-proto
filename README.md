# rm-proto

A `no_std`, allocation-free Rust implementation of the RoboMaster referee system and vision link communication protocols.

## Crates

| Crate | Description |
| --- | --- |
| `rm-frame` | Core framing layer: CRC, encode/decode, error types |
| `rm-link-serial` | Referee system serial protocol messages |
| `rm-link-vision` | Vision link messages and RC receiver |

## Frame Format

```text
┌──────┬─────┬─────┬──────┬────────┬──────┬───────┐
│ SOF  │ LEN │ SEQ │ CRC8 │ CMD_ID │ DATA │ CRC16 │
├──────┼─────┼─────┼──────┼────────┼──────┼───────┤
│  1 B │ 2 B │ 1 B │  1 B │   2 B  │ N B  │  2 B  │
└──────┴─────┴─────┴──────┴────────┴──────┴───────┘
```

| Field | Details |
| --- | --- |
| SOF | Start of frame marker: `0xA5` |
| LEN | Payload length, little-endian `u16` |
| SEQ | Sequence number, wrapping `u8` |
| CRC8 | CRC8/MAXIM-DOW over `[SOF, LEN, SEQ]`, initial value `0xFF` |
| CMD_ID | Command ID, little-endian `u16` |
| DATA | Payload bytes |
| CRC16 | CRC16/CCITT-FALSE over the entire preceding frame, initial value `0xFFFF` |

Minimum frame size (empty payload): **9 bytes**.

---

## rm-frame

### The `Marshaler` Trait

Implement this trait to make a type encodable and decodable:

```rust
pub trait Marshaler: Sized {
    const CMD_ID: u16;
    const PAYLOAD_SIZE: u16;
    fn marshal(&self, dst: &mut [u8]) -> Result<usize, MarshalerError>;
    fn unmarshal(raw: &[u8]) -> Result<Self, MarshalerError>;
}
```

Implementing `Marshaler` automatically provides the `ImplMarshal` (encode-only) and `ImplUnMarshal` (decode-only) sub-traits. Types that only need one direction can implement these directly.

### `Messager`

The main entry point for sending and receiving frames:

```rust
let mut messager = Messager::new(0); // initial sequence number

// Encode a message into a buffer
let n = messager.pack(&msg, &mut buf)?;

// Decode a raw frame from a buffer
let (raw_frame, consumed) = messager.unpack(&buf)?;

// Decode directly into a typed message
let (msg, consumed) = messager.unmarshal::<MyMessage>(&buf)?;
```

### Error Types

| Type | When it occurs |
| --- | --- |
| `PackError` | Buffer too small, invalid payload size |
| `UnPackError` | Missing header, bad checksum, incomplete data |
| `MarshalerError` | CMD_ID mismatch, wrong data length |

`UnPackError::skip()` returns how many bytes to advance for re-synchronization.

### CRC Utilities

```rust
use rm_frame::{calc_dji8, calc_dji16};

let crc8  = calc_dji8(&data);
let crc16 = calc_dji16(&data);
```

---

## rm-link-serial

All referee system messages implement the full `Marshaler` trait (encode + decode).

### Messages

| Module | Type | CMD_ID | Rate / Trigger | Payload |
| --- | --- | --- | --- | --- |
| `states` | `GameStatus` | `0x0001` | 1 Hz | 11 B |
| `result` | `GameResult` | `0x0002` | On match end | 1 B |
| `health` | `GameRobotHP` | `0x0003` | 3 Hz | 16 B |
| `event` | `GameEvent` | `0x0101` | 1 Hz | 4 B |
| `warning` | `RefereeWarning` | `0x0104` | On penalty / 1 Hz | 3 B |
| `dart` | `DartInfo` | `0x0105` | 1 Hz | 3 B |
| `status` | `RobotStatus` | `0x0201` | 10 Hz | 13 B |
| `heat` | `PowerHeat` | `0x0202` | 10 Hz | 14 B |
| `pos` | `RobotPos` | `0x0203` | 1 Hz | 12 B |
| `buff` | `RobotBuff` | `0x0204` | 3 Hz | 8 B |
| `hurt` | `HurtData` | `0x0206` | On damage | 1 B |

### Usage

```rust
use rm_frame::Messager;
use rm_link_serial::status::RobotStatus;

let messager = Messager::new(0);
let (status, _): (RobotStatus, usize) = messager.unmarshal(&buf)?;
println!("HP: {}/{}", status.current_hp, status.maximum_hp);
```

---

## rm-link-vision

### `Custom2Robot` — CMD_ID `0x0302`, 30 bytes

A custom controller-to-robot command carrying 6 joint angles (`f32`) and a gripper state (`bool`). Decode-only (`ImplUnMarshal`).

```rust
use rm_frame::Messager;
use rm_link_vision::Custom2Robot;

let messager = Messager::new(0);
let (cmd, _): (Custom2Robot, usize) = messager.unmarshal(&buf)?;
let joints = cmd.get_joints(); // &[f32; 6]
let gripper = cmd.get_gripper(); // bool
```

### `RemoteControl` — DT7/DR16 RC Receiver, 21-byte frame

Decodes the DT7/DR16 RC receiver data stream. Uses its own framing (SOF `0xA953` + CRC16 tail), separate from the standard `Messager` format.

State is stored in `portable_atomic` atomics, making it safe to share across tasks without a mutex.

```rust
use rm_link_vision::RemoteControl;

let rc = RemoteControl::new();

// Call from your DMA/interrupt handler
rc.update(&raw_bytes)?;

// Read from any task
let ch = rc.right_horizontal(); // i16, roughly [-660, 660]
let sw = rc.switch();           // Switch::C / N / S
let w  = rc.keyboard_w();       // bool
let vx = rc.mouse_vx();         // i16
```

**Available inputs:**

| Category | Members |
| --- | --- |
| Analog channels | `right_horizontal`, `right_vertical`, `left_horizontal`, `left_vertical` |
| Switch | `switch` (`C` / `N` / `S`) |
| Buttons | `pause`, `left_fn`, `right_fn`, `trigger`, `wheel` |
| Mouse | `mouse_vx/vy/vz`, `left/mid/right_button` |
| Keyboard | `W S A D Shift Ctrl Q E R F G Z X C V B` |

---

## Feature Flags

| Feature | Crates | Effect |
| --- | --- | --- |
| `defmt` | all | Derives `defmt::Format` on error types and key enums for embedded structured logging |

---

## `no_std` Support

All crates are `#![no_std]` with no heap allocation. They can be used directly in bare-metal and RTOS environments.

## Examples

```rust
static SIG_BUFFER: Signal<RawMutex, ([u8; 64], usize)> = Signal::new();
pub static RC: RemoteControl = unsafe { RemoteControl::const_new() };

#[embassy_executor::task]
pub async fn task(s: embassy_executor::SendSpawner, p: PictransSrc) -> ! {
    let mut config = Config::default();
    config.baudrate = 921600;
    config.data_bits = DataBits::DataBits8;
    config.parity = Parity::ParityNone;
    config.stop_bits = StopBits::STOP1;

    // Note: Config is valid, so Unwrap is safe.
    let pt = UartRx::new(p.uart_p, Irqs, p.uart_rx, p.dma_rx, config).unwrap();

    let mut dma_buf = [0u8; 256];
    let mut pt = pt.into_ring_buffered(&mut dma_buf);

    let mut buffer = [0u8; 64];

    s.must_spawn(handler());

    loop {
        match pt.read(&mut buffer).await {
            Ok(x) if x > 0 => {
                SIG_BUFFER.signal((buffer, x));
            }

            Ok(_) => {
                // No data received
            }

            Err(e) => {
                defmt::error!("RC Read Error: {:?}", e);
            }
        };
    }
}

#[embassy_executor::task]
async fn handler() -> ! {
    let mut t = utils::init_ticker!(1, ms);

    let msger = Messager::<DjiValidator>::new(0);

    let mut data: _ = Vec::<u8, { 64 * 10 }>::new();

    loop {
        match select(t.next(), SIG_BUFFER.wait()).await {
            Either::First(_) => t.reset(),
            Either::Second((buffer, len)) => {
                if let Err(_) = data.extend_from_slice(&buffer[..len]) {
                    defmt::warn!("Data Overflow, clearing buffer");
                    data.clear();
                    continue;
                }
            }
        }

        match msger.unpack(&data) {
            Ok((frame, consumed)) => {
                match frame.cmd_id() {
                    Custom2Robot::CMD_ID => match frame.unmarshal::<Custom2Robot>() {
                        Ok(x) => {
                            do_sth_ext((x.get_joints(), x.get_gripper()));
                        }
                        Err(e) => {
                            defmt::warn!("Custom2Robot Error: {:?}", e);
                        }
                    },

                    x => {
                        defmt::warn!("Unknown CMD_ID: 0x{:04X}", x);
                    }
                }

                let _ = data.drain(..consumed);
            }

            Err(Error::MissingHeader { skip } | Error::ReSync { skip }) => {
                match RC.update(&data) {
                    Ok(x) => {
                        Device::Controller.feed();
                        let _ = data.drain(..x);
                    }

                    Err(e) => {
                        let skip = skip.min(e.skip());
                        let _ = data.drain(..skip);
                    }
                };
            }

            Err(e) => {
                let _ = data.drain(..e.skip());
            }
        }
    }
}
```

---

## Reference

This library implements the **RoboMaster University Series 2026 Communication Protocol V1.2.0** (2026-02-09).
