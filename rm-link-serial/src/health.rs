use crate::private::*;

/// 机器人血量数据，固定以 3Hz 频率发送
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GameRobotHP {
    ally_1: u16,
    ally_2: u16,
    ally_3: u16,
    ally_4: u16,
    _reserved: u16,
    ally_7: u16,
    ally_outpost: u16,
    ally_base: u16,
}

impl GameRobotHP {
    pub const fn new(
        ally_1: u16,
        ally_2: u16,
        ally_3: u16,
        ally_4: u16,
        ally_7: u16,
        ally_outpost: u16,
        ally_base: u16,
    ) -> GameRobotHP {
        Self {
            ally_1,
            ally_2,
            ally_3,
            ally_4,
            _reserved: 0,
            ally_7,
            ally_outpost,
            ally_base,
        }
    }
}

impl GameRobotHP {
    /// 己方 1 号英雄机器人血量，若该机器人未上场或者被罚下，则血量为 0
    pub const fn ally1_hp(&self) -> u16 {
        self.ally_1
    }

    /// 己方 2 号工程机器人血量，若该机器人未上场或者被罚下，则血量为 0
    pub const fn ally2_hp(&self) -> u16 {
        self.ally_2
    }

    /// 己方 3 号步兵机器人血量，若该机器人未上场或者被罚下，则血量为 0
    pub const fn ally3_hp(&self) -> u16 {
        self.ally_3
    }

    /// 己方 4 号步兵机器人血量，若该机器人未上场或者被罚下，则血量为 0
    pub const fn ally4_hp(&self) -> u16 {
        self.ally_4
    }

    /// 己方 7 号哨兵机器人血量，若该机器人未上场或者被罚下，则血量为 0
    pub const fn ally7_hp(&self) -> u16 {
        self.ally_7
    }

    /// 己方前哨站血量，若前哨站未上场或者被罚下，则血量为 0
    pub const fn outpost_hp(&self) -> u16 {
        self.ally_outpost
    }

    /// 己方基地血量，若基地未上场或者被罚下，则血量为 0
    pub const fn base_hp(&self) -> u16 {
        self.ally_base
    }
}

impl Marshaler for GameRobotHP {
    const CMD_ID: u16 = 0x0003;
    const PAYLOAD_SIZE: u16 = 16;

    fn marshal(&self, dst: &mut [u8]) -> Result<usize> {
        dst[0..2].copy_from_slice(&self.ally_1.to_le_bytes());
        dst[2..4].copy_from_slice(&self.ally_2.to_le_bytes());
        dst[4..6].copy_from_slice(&self.ally_3.to_le_bytes());
        dst[6..8].copy_from_slice(&self.ally_4.to_le_bytes());
        dst[8..10].copy_from_slice(&self._reserved.to_le_bytes());
        dst[10..12].copy_from_slice(&self.ally_7.to_le_bytes());
        dst[12..14].copy_from_slice(&self.ally_outpost.to_le_bytes());
        dst[14..16].copy_from_slice(&self.ally_base.to_le_bytes());

        Ok(Self::PAYLOAD_SIZE as usize)
    }

    fn unmarshal(raw: &[u8]) -> Result<Self> {
        let ally_1 = u16::from_le_bytes([raw[0], raw[1]]);
        let ally_2 = u16::from_le_bytes([raw[2], raw[3]]);
        let ally_3 = u16::from_le_bytes([raw[4], raw[5]]);
        let ally_4 = u16::from_le_bytes([raw[6], raw[7]]);
        let _reserved = u16::from_le_bytes([raw[8], raw[9]]);
        let ally_7 = u16::from_le_bytes([raw[10], raw[11]]);
        let ally_outpost = u16::from_le_bytes([raw[12], raw[13]]);
        let ally_base = u16::from_le_bytes([raw[14], raw[15]]);

        Ok(Self {
            ally_1,
            ally_2,
            ally_3,
            ally_4,
            _reserved,
            ally_7,
            ally_outpost,
            ally_base,
        })
    }
}

#[cfg(test)]
#[test]
fn test() {
    const SIZE: usize = GameRobotHP::PAYLOAD_SIZE as usize;

    let status = GameRobotHP {
        ally_1: 1000,
        ally_2: 2000,
        ally_3: 3000,
        ally_4: 4000,
        _reserved: 0,
        ally_7: 7000,
        ally_outpost: 8000,
        ally_base: 9000,
    };

    let mut buf = [0u8; SIZE + 10];
    let sz = status.marshal(&mut buf).unwrap();
    assert_eq!(sz, SIZE);

    let decoded = GameRobotHP::unmarshal(&buf[..SIZE]).unwrap();
    assert_eq!(decoded.ally1_hp(), 1000);
    assert_eq!(decoded.ally2_hp(), 2000);
    assert_eq!(decoded.ally3_hp(), 3000);
    assert_eq!(decoded.ally4_hp(), 4000);
    assert_eq!(decoded.ally7_hp(), 7000);
    assert_eq!(decoded.outpost_hp(), 8000);
    assert_eq!(decoded.base_hp(), 9000);
}
