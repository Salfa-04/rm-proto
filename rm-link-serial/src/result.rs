use crate::private::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Winner {
    /// 平局
    Draw = 0,
    /// 红方胜利
    Red = 1,
    /// 蓝方胜利
    Blue = 2,
}

/// 比赛结果数据，比赛结束触发发送
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GameResult {
    winner: Winner,
}

impl GameResult {
    pub const fn new(winner: Winner) -> GameResult {
        Self { winner }
    }
}

impl GameResult {
    /// | 值 | 含义 |
    /// |---|------|
    /// | 0 | 平局 |
    /// | 1 | 红方胜利 |
    /// | 2 | 蓝方胜利 |
    pub const fn winner(&self) -> Winner {
        self.winner
    }
}

impl Marshaler for GameResult {
    const CMD_ID: u16 = 0x0002;
    const PAYLOAD_SIZE: u16 = 1;

    fn marshal(&self, dst: &mut [u8]) -> Result<usize> {
        dst[0] = self.winner as u8;

        Ok(Self::PAYLOAD_SIZE as usize)
    }

    fn unmarshal(raw: &[u8]) -> Result<Self> {
        let winner = match raw[0] {
            0 => Winner::Draw,
            1 => Winner::Red,
            2 => Winner::Blue,

            _ => return Err((0, "Invalid Winner").into()),
        };

        Ok(GameResult { winner })
    }
}

#[cfg(test)]
#[test]
fn test() {
    const SIZE: usize = GameResult::PAYLOAD_SIZE as usize;

    let status = GameResult {
        winner: Winner::Blue,
    };

    let mut buf = [0u8; SIZE + 10];
    let sz = status.marshal(&mut buf).unwrap();
    assert_eq!(sz, SIZE);

    let decoded = GameResult::unmarshal(&buf[..SIZE]).unwrap();
    assert_eq!(decoded.winner(), Winner::Blue);
}
