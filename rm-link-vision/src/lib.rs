#![cfg_attr(not(test), no_std)]

pub use custom::Custom2Robot;
pub use remote::{RemoteControl, Switch};

/// 0x0302 - Custom to Robot
mod custom;

/// Receiver to Controlled Robot
mod remote;

mod private {
    pub type Result<T, E = Error> = core::result::Result<T, E>;
    pub use portable_atomic::{AtomicU16, AtomicU64, Ordering::Relaxed};
    pub use rm_frame::{ImplCommandMsg, ImplUnMarshal};
    pub use rm_frame::{MarshalerError as Error, UnPackError, calc_dji16};
}

#[cfg(test)]
#[test]
fn test_command_id() {
    use crate::private::ImplCommandMsg;

    assert_eq!(custom::Custom2Robot::CMD_ID, 0x0302);
    assert_eq!(custom::Custom2Robot::PAYLOAD_SIZE, 30);
}
