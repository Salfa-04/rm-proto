use crate::private::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GameType {
    /// RoboMaster 机甲大师超级对抗赛
    RMUC = 1,
    /// RoboMaster 机甲大师高校单项赛
    RMUT = 2,
    /// ICRA RoboMaster 高校人工智能挑战赛
    RMUA = 3,
    /// RoboMaster 机甲大师高校联盟赛（3V3 对抗）
    RMUL3V3 = 4,
    /// RoboMaster 机甲大师高校联盟赛（1V1 步兵对抗）
    RMUL1V1 = 5,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GameProgress {
    /// 未开始比赛
    NotStarted = 0,
    /// 准备阶段
    Prepared = 1,
    /// 十五秒裁判系统自检阶段
    SelfCheck = 2,
    /// 五秒倒计时
    CountDown5s = 3,
    /// 比赛进行中
    InProgress = 4,
    /// 比赛结算中
    Calculating = 5,
}

/// 比赛状态数据，固定以 1Hz 频率发送
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GameStatus {
    game_type: GameType,
    game_progress: GameProgress,
    remaining_time_s: u16,
    unix_timestamp: u64,
}

impl GameStatus {
    pub const fn new(
        game_type: GameType,
        game_progress: GameProgress,
        remaining_time_s: u16,
        unix_timestamp: u64,
    ) -> GameStatus {
        Self {
            game_type,
            game_progress,
            remaining_time_s,
            unix_timestamp,
        }
    }
}

impl GameStatus {
    /// 比赛类型
    ///
    /// | 值 | 类型 | 说明 |
    /// |---:|---|---|
    /// | 1 | RMUC | RoboMaster 机甲大师超级对抗赛 |
    /// | 2 | RMUT | RoboMaster 机甲大师高校单项赛 |
    /// | 3 | RMUA | ICRA RoboMaster 高校人工智能挑战赛 |
    /// | 4 | RMUL3V3 | RoboMaster 机甲大师高校联盟赛 3V3 对抗 |
    /// | 5 | RMUL1V1 | RoboMaster 机甲大师高校联盟赛步兵对抗 |
    pub const fn game_type(&self) -> GameType {
        self.game_type
    }

    /// 当前比赛阶段
    ///
    /// | 值 | 阶段 | 说明 |
    /// |---:|---|---|
    /// | 0 | NotStarted | 未开始比赛 |
    /// | 1 | PrePared | 准备阶段 |
    /// | 2 | SelfCheck | 十五秒裁判系统自检阶段 |
    /// | 3 | CountDown5s | 五秒倒计时 |
    /// | 4 | InProgress | 比赛中 |
    /// | 5 | Calculating | 比赛结算中 |
    pub const fn game_progress(&self) -> GameProgress {
        self.game_progress
    }

    /// 当前阶段剩余时间，单位：秒
    pub const fn remaining_time_s(&self) -> u16 {
        self.remaining_time_s
    }

    /// UNIX 时间，当机器人正确连接到裁判系统的 NTP 服务器后生效
    pub const fn unix_timestamp(&self) -> u64 {
        self.unix_timestamp
    }
}

impl Marshaler for GameStatus {
    const CMD_ID: u16 = 0x0001;
    const PAYLOAD_SIZE: u16 = 11;

    fn marshal(&self, dst: &mut [u8]) -> Result<usize> {
        dst[0] = ((self.game_type as u8) & 0xF) | (((self.game_progress as u8) & 0xF) << 4);
        dst[1..3].copy_from_slice(&self.remaining_time_s.to_le_bytes());
        dst[3..11].copy_from_slice(&self.unix_timestamp.to_le_bytes());

        Ok(Self::PAYLOAD_SIZE as usize)
    }

    fn unmarshal(raw: &[u8]) -> Result<Self> {
        let game_type = match raw[0] & 0xF {
            1 => GameType::RMUC,
            2 => GameType::RMUT,
            3 => GameType::RMUA,
            4 => GameType::RMUL3V3,
            5 => GameType::RMUL1V1,

            _ => return Err((0, "Invalid GameType").into()),
        };

        let game_progress = match (raw[0] >> 4) & 0xF {
            0 => GameProgress::NotStarted,
            1 => GameProgress::Prepared,
            2 => GameProgress::SelfCheck,
            3 => GameProgress::CountDown5s,
            4 => GameProgress::InProgress,
            5 => GameProgress::Calculating,

            _ => return Err((1, "Invalid GameProgress").into()),
        };

        let remaining_time_s = u16::from_le_bytes([raw[1], raw[2]]);
        let unix_timestamp = u64::from_le_bytes([
            raw[3], raw[4], raw[5], raw[6], raw[7], raw[8], raw[9], raw[10],
        ]);

        Ok(GameStatus {
            game_type,
            game_progress,
            remaining_time_s,
            unix_timestamp,
        })
    }
}

#[cfg(test)]
#[test]
fn test() {
    const SIZE: usize = GameStatus::PAYLOAD_SIZE as usize;

    let status = GameStatus {
        game_type: GameType::RMUA,
        game_progress: GameProgress::InProgress,
        remaining_time_s: 1234,
        unix_timestamp: 1672531199,
    };

    let mut buf = [0u8; SIZE + 10];
    let sz = status.marshal(&mut buf).unwrap();
    assert_eq!(sz, SIZE);

    let decoded = GameStatus::unmarshal(&buf[..SIZE]).unwrap();
    assert_eq!(decoded.game_type(), GameType::RMUA);
    assert_eq!(decoded.game_progress(), GameProgress::InProgress);
    assert_eq!(decoded.remaining_time_s(), 1234);
    assert_eq!(decoded.unix_timestamp(), 1672531199);
}
