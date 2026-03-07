use crate::private::*;

/// 实时底盘缓冲能量和射击热量数据，固定以 10Hz 频率发送
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PowerHeat {
    _reserved_1: u16,
    _reserved_2: u16,
    _reserved_3: u32,
    buffer_energy: u16,
    shooter_heat_17mm: u16,
    shooter_heat_42mm: u16,
}

impl PowerHeat {
    pub const fn new(
        buffer_energy: u16,
        shooter_heat_17mm: u16,
        shooter_heat_42mm: u16,
    ) -> PowerHeat {
        Self {
            _reserved_1: 0,
            _reserved_2: 0,
            _reserved_3: 0,
            buffer_energy,
            shooter_heat_17mm,
            shooter_heat_42mm,
        }
    }
}

impl PowerHeat {
    /// 缓冲能量（单位：J）
    pub const fn buffer_energy(&self) -> u16 {
        self.buffer_energy
    }

    /// 17mm 发射机构的射击热量
    pub const fn shooter_heat_17mm(&self) -> u16 {
        self.shooter_heat_17mm
    }

    /// 42mm 发射机构的射击热量
    pub const fn shooter_heat_42mm(&self) -> u16 {
        self.shooter_heat_42mm
    }
}

impl Marshaler for PowerHeat {
    const CMD_ID: u16 = 0x0202;
    const PAYLOAD_SIZE: u16 = 14;

    fn marshal(&self, dst: &mut [u8]) -> Result<usize> {
        dst[0..2].copy_from_slice(&self._reserved_1.to_le_bytes());
        dst[2..4].copy_from_slice(&self._reserved_2.to_le_bytes());
        dst[4..8].copy_from_slice(&self._reserved_3.to_le_bytes());
        dst[8..10].copy_from_slice(&self.buffer_energy.to_le_bytes());
        dst[10..12].copy_from_slice(&self.shooter_heat_17mm.to_le_bytes());
        dst[12..14].copy_from_slice(&self.shooter_heat_42mm.to_le_bytes());

        Ok(Self::PAYLOAD_SIZE as usize)
    }

    fn unmarshal(raw: &[u8]) -> Result<Self> {
        let _reserved_1 = u16::from_le_bytes([raw[0], raw[1]]);
        let _reserved_2 = u16::from_le_bytes([raw[2], raw[3]]);
        let _reserved_3 = u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]);
        let buffer_energy = u16::from_le_bytes([raw[8], raw[9]]);
        let shooter_heat_17mm = u16::from_le_bytes([raw[10], raw[11]]);
        let shooter_heat_42mm = u16::from_le_bytes([raw[12], raw[13]]);

        Ok(PowerHeat {
            _reserved_1,
            _reserved_2,
            _reserved_3,
            buffer_energy,
            shooter_heat_17mm,
            shooter_heat_42mm,
        })
    }
}

#[cfg(test)]
#[test]
fn test() {
    const SIZE: usize = PowerHeat::PAYLOAD_SIZE as usize;

    let status = PowerHeat {
        _reserved_1: 0,
        _reserved_2: 0,
        _reserved_3: 0,
        buffer_energy: 1234,
        shooter_heat_17mm: 2345,
        shooter_heat_42mm: 3456,
    };

    let mut buf = [0u8; SIZE + 10];
    let sz = status.marshal(&mut buf).unwrap();
    assert_eq!(sz, SIZE);

    let decoded = PowerHeat::unmarshal(&buf[..SIZE]).unwrap();
    assert_eq!(decoded.buffer_energy(), 1234);
    assert_eq!(decoded.shooter_heat_17mm(), 2345);
    assert_eq!(decoded.shooter_heat_42mm(), 3456);
}
