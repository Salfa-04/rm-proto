use crate::private::*;

/// 机器人增益和底盘能量数据，固定以 3Hz 频率发送
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RobotBuff {
    recovery_rate: u8,
    cooling_value: u16,
    defence_rate: u8,
    vulnerability_rate: u8,
    attack_rate: u16,
    remain_energy: u8,
}

impl RobotBuff {
    pub const fn new(
        recovery_rate: u8,
        cooling_value: u16,
        defence_rate: u8,
        vulnerability_rate: u8,
        attack_rate: u16,
        remain_energy: u8,
    ) -> RobotBuff {
        Self {
            recovery_rate,
            cooling_value,
            defence_rate,
            vulnerability_rate,
            attack_rate,
            remain_energy,
        }
    }
}

impl RobotBuff {
    /// 机器人回血增益（百分比，值为 10 表示每秒恢复血量上限的 10%）
    pub const fn recovery_rate(&self) -> u8 {
        self.recovery_rate
    }

    /// 机器人射击热量冷却增益具体值（直接值，值为 x 表示热量冷却增加 x/s）
    pub const fn cooling_value(&self) -> u16 {
        self.cooling_value
    }

    /// 机器人防御增益（百分比，值为 50 表示 50%防御增益）
    pub const fn defence_rate(&self) -> u8 {
        self.defence_rate
    }

    /// 机器人负防御增益（百分比，值为 30 表示-30%防御增益）
    pub const fn vulnerability_rate(&self) -> u8 {
        self.vulnerability_rate
    }

    /// 机器人攻击增益（百分比，值为 50 表示 50%攻击增益）
    pub const fn attack_rate(&self) -> u16 {
        self.attack_rate
    }

    /// bit 0-6：机器人剩余能量值反馈，以 16 进制标识机器人剩余能量值比例。
    /// 仅在机器人剩余能量小于 50% 时反馈，其余默认反馈 `0x80`。机器人初始能量视为 100%。
    ///
    /// | bit 位 | 条件 | 取值 |
    /// |---|---|---|
    /// | bit 0 | 剩余能量 ≥ 125% | 1（否则为 0） |
    /// | bit 1 | 剩余能量 ≥ 100% | 1（否则为 0） |
    /// | bit 2 | 剩余能量 ≥ 50%  | 1（否则为 0） |
    /// | bit 3 | 剩余能量 ≥ 30%  | 1（否则为 0） |
    /// | bit 4 | 剩余能量 ≥ 15%  | 1（否则为 0） |
    /// | bit 5 | 剩余能量 ≥ 5%   | 1（否则为 0） |
    /// | bit 6 | 剩余能量 ≥ 1%   | 1（否则为 0） |
    pub const fn remain_energy(&self) -> u8 {
        self.remain_energy
    }
}

impl Marshaler for RobotBuff {
    const CMD_ID: u16 = 0x0204;
    const PAYLOAD_SIZE: u16 = 8;

    fn marshal(&self, dst: &mut [u8]) -> Result<usize> {
        dst[0] = self.recovery_rate;
        dst[1..3].copy_from_slice(&self.cooling_value.to_le_bytes());
        dst[3] = self.defence_rate;
        dst[4] = self.vulnerability_rate;
        dst[5..7].copy_from_slice(&self.attack_rate.to_le_bytes());
        dst[7] = self.remain_energy;

        Ok(Self::PAYLOAD_SIZE as usize)
    }

    fn unmarshal(raw: &[u8]) -> Result<Self> {
        let recovery_rate = raw[0];
        let cooling_value = u16::from_le_bytes([raw[1], raw[2]]);
        let defence_rate = raw[3];
        let vulnerability_rate = raw[4];
        let attack_rate = u16::from_le_bytes([raw[5], raw[6]]);
        let remain_energy = raw[7];

        Ok(RobotBuff {
            recovery_rate,
            cooling_value,
            defence_rate,
            vulnerability_rate,
            attack_rate,
            remain_energy,
        })
    }
}

#[cfg(test)]
#[test]
fn test() {
    const SIZE: usize = RobotBuff::PAYLOAD_SIZE as usize;

    let buff = RobotBuff {
        recovery_rate: 10,
        cooling_value: 200,
        defence_rate: 5,
        vulnerability_rate: 3,
        attack_rate: 1500,
        remain_energy: 80,
    };

    let mut buf = [0u8; SIZE + 10];
    let sz = buff.marshal(&mut buf).unwrap();
    assert_eq!(sz, SIZE);

    let buff2 = RobotBuff::unmarshal(&buf[..SIZE]).unwrap();
    assert_eq!(buff2.recovery_rate(), 10);
    assert_eq!(buff2.cooling_value(), 200);
    assert_eq!(buff2.defence_rate(), 5);
    assert_eq!(buff2.vulnerability_rate(), 3);
    assert_eq!(buff2.attack_rate(), 1500);
    assert_eq!(buff2.remain_energy(), 80);
}
