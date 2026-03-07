use crate::private::*;

/// 机器人性能体系数据，固定以 10Hz 频率发送
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RobotStatus {
    robot_id: u8,
    robot_level: u8,
    current_hp: u16,
    maximum_hp: u16,
    heat_cooling_down: u16,
    shooter_heat_limit: u16,
    chassis_power_limit: u16,
    power_output: u8,
}

impl RobotStatus {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        robot_id: u8,
        robot_level: u8,
        current_hp: u16,
        maximum_hp: u16,
        heat_cooling_down: u16,
        shooter_heat_limit: u16,
        chassis_power_limit: u16,
        power_output: u8,
    ) -> RobotStatus {
        Self {
            robot_id,
            robot_level,
            current_hp,
            maximum_hp,
            heat_cooling_down,
            shooter_heat_limit,
            chassis_power_limit,
            power_output,
        }
    }
}

impl RobotStatus {
    /// 本机器人 ID
    pub const fn robot_id(&self) -> u8 {
        self.robot_id
    }

    /// 本机器人等级
    pub const fn robot_level(&self) -> u8 {
        self.robot_level
    }

    /// 本机器人当前血量
    pub const fn current_hp(&self) -> u16 {
        self.current_hp
    }

    /// 本机器人血量上限
    pub const fn maximum_hp(&self) -> u16 {
        self.maximum_hp
    }

    /// 本机器人射击热量每秒冷却值
    pub const fn heat_cooling_down(&self) -> u16 {
        self.heat_cooling_down
    }

    /// 本机器人射击热量上限
    pub const fn shooter_heat_limit(&self) -> u16 {
        self.shooter_heat_limit
    }

    /// 本机器人底盘功率上限
    pub const fn chassis_power_limit(&self) -> u16 {
        self.chassis_power_limit
    }

    /// 本机器人云台电源输出状态
    pub const fn gimbal_power_output(&self) -> bool {
        (self.power_output & (1 << 0)) != 0
    }

    /// 本机器人底盘电源输出状态
    pub const fn chassis_power_output(&self) -> bool {
        (self.power_output & (1 << 1)) != 0
    }

    /// 本机器人射击电源输出状态
    pub const fn shooter_power_output(&self) -> bool {
        (self.power_output & (1 << 2)) != 0
    }
}

impl Marshaler for RobotStatus {
    const CMD_ID: u16 = 0x0201;
    const PAYLOAD_SIZE: u16 = 13;

    fn marshal(&self, dst: &mut [u8]) -> Result<usize> {
        dst[0] = self.robot_id;
        dst[1] = self.robot_level;
        dst[2..4].copy_from_slice(&self.current_hp.to_le_bytes());
        dst[4..6].copy_from_slice(&self.maximum_hp.to_le_bytes());
        dst[6..8].copy_from_slice(&self.heat_cooling_down.to_le_bytes());
        dst[8..10].copy_from_slice(&self.shooter_heat_limit.to_le_bytes());
        dst[10..12].copy_from_slice(&self.chassis_power_limit.to_le_bytes());
        dst[12] = self.power_output;

        Ok(Self::PAYLOAD_SIZE as usize)
    }

    fn unmarshal(raw: &[u8]) -> Result<Self> {
        let robot_id = raw[0];
        let robot_level = raw[1];
        let current_hp = u16::from_le_bytes([raw[2], raw[3]]);
        let maximum_hp = u16::from_le_bytes([raw[4], raw[5]]);
        let heat_cooling_down = u16::from_le_bytes([raw[6], raw[7]]);
        let shooter_heat_limit = u16::from_le_bytes([raw[8], raw[9]]);
        let chassis_power_limit = u16::from_le_bytes([raw[10], raw[11]]);
        let power_output = raw[12];

        Ok(RobotStatus {
            robot_id,
            robot_level,
            current_hp,
            maximum_hp,
            heat_cooling_down,
            shooter_heat_limit,
            chassis_power_limit,
            power_output,
        })
    }
}

#[cfg(test)]
#[test]
fn test() {
    const SIZE: usize = RobotStatus::PAYLOAD_SIZE as usize;

    let state = RobotStatus {
        robot_id: 3,
        robot_level: 2,
        current_hp: 1500,
        maximum_hp: 2000,
        heat_cooling_down: 300,
        shooter_heat_limit: 500,
        chassis_power_limit: 800,
        power_output: 0b0000_0101,
    };

    let mut buf = [0u8; SIZE + 10];
    let sz = state.marshal(&mut buf).unwrap();
    assert_eq!(sz, SIZE);

    let decoded = RobotStatus::unmarshal(&buf[..SIZE]).unwrap();
    assert_eq!(decoded.robot_id(), 3);
    assert_eq!(decoded.robot_level(), 2);
    assert_eq!(decoded.current_hp(), 1500);
    assert_eq!(decoded.maximum_hp(), 2000);
    assert_eq!(decoded.heat_cooling_down(), 300);
    assert_eq!(decoded.shooter_heat_limit(), 500);
    assert_eq!(decoded.chassis_power_limit(), 800);
    assert_eq!(decoded.gimbal_power_output(), true);
    assert_eq!(decoded.chassis_power_output(), false);
    assert_eq!(decoded.shooter_power_output(), true);
}
