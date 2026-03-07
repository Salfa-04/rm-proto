use crate::private::*;

/// 机器人位置数据，固定以 1Hz 频率发送
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RobotPos {
    x: f32,
    y: f32,
    angle: f32,
}

impl RobotPos {
    pub const fn new(x: f32, y: f32, angle: f32) -> RobotPos {
        Self { x, y, angle }
    }
}

impl RobotPos {
    /// 本机器人位置 x 坐标，单位：m
    pub const fn pos_x(&self) -> f32 {
        self.x
    }

    /// 本机器人位置 y 坐标，单位：m
    pub const fn pos_y(&self) -> f32 {
        self.y
    }

    /// 本机器人测速模块的朝向，单位：度。正北为 0 度
    pub const fn angle(&self) -> f32 {
        self.angle
    }
}

impl Marshaler for RobotPos {
    const CMD_ID: u16 = 0x0203;
    const PAYLOAD_SIZE: u16 = 12;

    fn marshal(&self, dst: &mut [u8]) -> Result<usize> {
        dst[0..4].copy_from_slice(&self.x.to_le_bytes());
        dst[4..8].copy_from_slice(&self.y.to_le_bytes());
        dst[8..12].copy_from_slice(&self.angle.to_le_bytes());

        Ok(Self::PAYLOAD_SIZE as usize)
    }

    fn unmarshal(raw: &[u8]) -> Result<Self> {
        let x = f32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
        let y = f32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]);
        let angle = f32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]);

        Ok(RobotPos { x, y, angle })
    }
}

#[cfg(test)]
#[test]
fn test() {
    const SIZE: usize = RobotPos::PAYLOAD_SIZE as usize;

    let pos = RobotPos {
        x: 1.0,
        y: 2.0,
        angle: 3.0,
    };

    let mut buf = [0u8; SIZE + 10];
    let sz = pos.marshal(&mut buf).unwrap();
    assert_eq!(sz, SIZE);

    let pos2 = RobotPos::unmarshal(&buf[..SIZE]).unwrap();
    assert_eq!(pos2.pos_x(), 1.0);
    assert_eq!(pos2.pos_y(), 2.0);
    assert_eq!(pos2.angle(), 3.0);
}
