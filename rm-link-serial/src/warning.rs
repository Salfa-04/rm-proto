use crate::private::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    /// 双方黄牌（双方均触发警告，违规机器人 ID 为 `0`）
    YellowCardBoth = 1,
    /// 己方黄牌
    YellowCard = 2,
    /// 己方红牌
    RedCard = 3,
    /// 己方判负（违规机器人 ID 为 `0`）
    Loss = 4,
}

/// 裁判警告数据，己方判罚/判负时触发发送，其余时间以 1Hz 频率发送
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RefereeWarning {
    level: Level,
    robot_id: u8,
    count: u8,
}

impl RefereeWarning {
    pub const fn new(level: Level, robot_id: u8, count: u8) -> RefereeWarning {
        Self {
            level,
            robot_id,
            count,
        }
    }
}

impl RefereeWarning {
    /// 己方最后一次受到判罚的等级：
    ///
    /// | 值 | 含义     |
    /// |---:|----------|
    /// | 1  | 双方黄牌 |
    /// | 2  | 黄牌     |
    /// | 3  | 红牌     |
    /// | 4  | 判负     |
    pub const fn level(&self) -> Level {
        self.level
    }

    /// 己方最后一次受到判罚的违规机器人 ID。
    ///
    /// 例如：红方 1 号机器人 ID 为 `1`，蓝方 1 号机器人 ID 为 `101`。
    /// 当判负或双方黄牌时，该值为 `0`。
    pub const fn robot_id(&self) -> u8 {
        self.robot_id
    }

    /// 己方最后一次受到判罚的违规机器人对应判罚等级的违规次数。（开局默认为 0。）
    pub const fn count(&self) -> u8 {
        self.count
    }
}

impl Marshaler for RefereeWarning {
    const CMD_ID: u16 = 0x0104;
    const PAYLOAD_SIZE: u16 = 3;

    fn marshal(&self, dst: &mut [u8]) -> Result<usize> {
        dst[0] = self.level as u8;
        dst[1] = self.robot_id;
        dst[2] = self.count;

        Ok(Self::PAYLOAD_SIZE as usize)
    }

    fn unmarshal(raw: &[u8]) -> Result<Self> {
        let level = match raw[0] {
            1 => Level::YellowCardBoth,
            2 => Level::YellowCard,
            3 => Level::RedCard,
            4 => Level::Loss,

            _ => return Err((0, "Invalid Level").into()),
        };

        let robot_id = raw[1];
        let count = raw[2];

        Ok(RefereeWarning {
            level,
            robot_id,
            count,
        })
    }
}

#[cfg(test)]
#[test]
fn test() {
    const SIZE: usize = RefereeWarning::PAYLOAD_SIZE as usize;

    let warning = RefereeWarning {
        level: Level::RedCard,
        robot_id: 5,
        count: 2,
    };

    let mut buf = [0u8; SIZE + 10];
    let sz = warning.marshal(&mut buf).unwrap();
    assert_eq!(sz, SIZE);

    let decoded = RefereeWarning::unmarshal(&buf[..SIZE]).unwrap();
    assert_eq!(decoded.level(), Level::RedCard);
    assert_eq!(decoded.robot_id(), 5);
    assert_eq!(decoded.count(), 2);
}
