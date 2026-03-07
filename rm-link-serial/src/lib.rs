//! RoboMaster 裁判系统串口协议消息类型库。
//!
//! 本库将 RoboMaster 裁判系统串口通信协议中的各消息结构映射为 Rust 类型，
//! 每个类型均实现 [`rm_frame::Marshaler`] trait，可直接配合 [`rm_frame::Messager`]
//! 完成帧的编解码。
//!
//! # 支持的消息类型
//!
//! | CMD_ID   | 模块        | 类型                        | 频率 / 触发条件   |
//! |----------|-------------|-----------------------------|-------------------|
//! | `0x0001` | [`states`]  | [`states::GameStatus`]      | 1 Hz              |
//! | `0x0002` | [`result`]  | [`result::GameResult`]      | 比赛结束时触发    |
//! | `0x0003` | [`health`]  | [`health::GameRobotHP`]     | 3 Hz              |
//! | `0x0101` | [`event`]   | [`event::GameEvent`]        | 1 Hz              |
//! | `0x0104` | [`warning`] | [`warning::RefereeWarning`] | 判罚时触发 / 1 Hz |
//! | `0x0105` | [`dart`]    | [`dart::DartInfo`]          | 1 Hz              |
//! | `0x0201` | [`status`]  | [`status::RobotStatus`]     | 10 Hz             |
//! | `0x0202` | [`heat`]    | [`heat::PowerHeat`]         | 10 Hz             |
//! | `0x0203` | [`pos`]     | [`pos::RobotPos`]           | 1 Hz              |
//! | `0x0204` | [`buff`]    | [`buff::RobotBuff`]         | 3 Hz              |
//! | `0x0206` | [`hurt`]    | [`hurt::HurtData`]          | 伤害发生时触发    |
//!
//! # 使用示例
//!
//! ```rust,ignore
//! use rm_frame::Messager;
//! use rm_link_serial::{status::RobotStatus, heat::PowerHeat};
//!
//! let messager = Messager::new(0);
//!
//! // 从串口接收到一帧原始字节后解帧
//! let (frame, _consumed) = messager.unpack(&serial_buf)?;
//!
//! // 根据 CMD_ID 分发并解码为具体类型
//! match frame.cmd_id() {
//!     0x0201 => {
//!         let msg: RobotStatus = frame.unmarshal()?;
//!         let hp = msg.current_hp();
//!     }
//!     0x0202 => {
//!         let msg: PowerHeat = frame.unmarshal()?;
//!         let energy = msg.buffer_energy();
//!     }
//!     _ => {}
//! }
//! ```
//!
//! # 协议参考
//!
//! 官方文档：[RMU Communication Protocol](https://bbs.robomaster.com/wiki/20204847/811363)

#![cfg_attr(not(test), no_std)]

/// `0x0001` — 比赛状态，固定以 1 Hz 频率发送
pub mod states;

/// `0x0002` — 比赛结果，比赛结束时触发发送
pub mod result;

/// `0x0003` — 己方机器人血量，固定以 3 Hz 频率发送
pub mod health;

/// `0x0101` — 场地事件，固定以 1 Hz 频率发送
pub mod event;

/// `0x0104` — 裁判警告，己方受判罚时触发，其余时间以 1 Hz 频率发送
pub mod warning;

/// `0x0105` — 飞镖发射信息，固定以 1 Hz 频率发送
pub mod dart;

/// `0x0201` — 机器人性能体系数据，固定以 10 Hz 频率发送
pub mod status;

/// `0x0202` — 底盘缓冲能量与射击热量，固定以 10 Hz 频率发送
pub mod heat;

/// `0x0203` — 机器人位置与朝向，固定以 1 Hz 频率发送
pub mod pos;

/// `0x0204` — 机器人增益与底盘能量，固定以 3 Hz 频率发送
pub mod buff;

/// `0x0206` — 伤害状态，伤害发生后触发发送
pub mod hurt;

mod private {
    pub type Result<T> = core::result::Result<T, Error>;
    pub use rm_frame::{Marshaler, MarshalerError as Error};
}

#[cfg(test)]
#[test]
fn test_command_msg() {
    use crate::private::Marshaler;

    assert_eq!(states::GameStatus::CMD_ID, 0x0001);
    assert_eq!(states::GameStatus::PAYLOAD_SIZE, 11);

    assert_eq!(result::GameResult::CMD_ID, 0x0002);
    assert_eq!(result::GameResult::PAYLOAD_SIZE, 1);

    assert_eq!(health::GameRobotHP::CMD_ID, 0x0003);
    assert_eq!(health::GameRobotHP::PAYLOAD_SIZE, 16);

    assert_eq!(event::GameEvent::CMD_ID, 0x0101);
    assert_eq!(event::GameEvent::PAYLOAD_SIZE, 4);

    assert_eq!(warning::RefereeWarning::CMD_ID, 0x0104);
    assert_eq!(warning::RefereeWarning::PAYLOAD_SIZE, 3);

    assert_eq!(dart::DartInfo::CMD_ID, 0x0105);
    assert_eq!(dart::DartInfo::PAYLOAD_SIZE, 3);

    assert_eq!(status::RobotStatus::CMD_ID, 0x0201);
    assert_eq!(status::RobotStatus::PAYLOAD_SIZE, 13);

    assert_eq!(heat::PowerHeat::CMD_ID, 0x0202);
    assert_eq!(heat::PowerHeat::PAYLOAD_SIZE, 14);

    assert_eq!(pos::RobotPos::CMD_ID, 0x0203);
    assert_eq!(pos::RobotPos::PAYLOAD_SIZE, 12);

    assert_eq!(buff::RobotBuff::CMD_ID, 0x0204);
    assert_eq!(buff::RobotBuff::PAYLOAD_SIZE, 8);

    assert_eq!(hurt::HurtData::CMD_ID, 0x0206);
    assert_eq!(hurt::HurtData::PAYLOAD_SIZE, 1);
}
