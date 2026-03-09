#![cfg_attr(not(test), no_std)]

pub use custom::Custom2Robot;

/// 0x0302 - Custom to Robot
mod custom;

mod private {
    pub type Result<T, E = Error> = core::result::Result<T, E>;
    pub use rm_frame::MarshalerError as Error;
    pub use rm_frame::{ImplCommandMsg, ImplUnMarshal};
}

#[cfg(test)]
#[test]
fn test_command_id() {
    use crate::private::ImplCommandMsg;

    assert_eq!(custom::Custom2Robot::CMD_ID, 0x0302);
    assert_eq!(custom::Custom2Robot::PAYLOAD_SIZE, 30);
}
