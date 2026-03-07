use crate::private::*;

/// 场地事件数据，固定以 1Hz 频率发送
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GameEvent {
    event_data: u32,
}

impl GameEvent {
    pub const fn new(event_data: u32) -> GameEvent {
        Self { event_data }
    }
}

impl GameEvent {
    /// | 位段 | 字段 | 取值说明 | 备注 |
    /// |---|---|---|---|
    /// | bit 0 | 己方与资源区不重叠的补给区占领状态 | 0=未占领，1=已占领 |  |
    /// | bit 1 | 己方与资源区重叠的补给区占领状态 | 0=未占领，1=已占领 |  |
    /// | bit 2 | 己方补给区占领状态 | 0=未占领，1=已占领 | 仅 RMUL 适用 |
    /// | bit 3-4 | 己方小能量机关激活状态 | 0=未激活，1=已激活，2=正在激活 |  |
    /// | bit 5-6 | 己方大能量机关激活状态 | 0=未激活，1=已激活，2=正在激活 |  |
    /// | bit 7-8 | 己方中央高地占领状态 | 1=被己方占领，2=被对方占领 |  |
    /// | bit 9-10 | 己方梯形高地占领状态 | 1=已占领 |  |
    /// | bit 11-19 | 对方飞镖最后一次击中己方前哨站或基地的时间 | 0~420（开局默认 0） |  |
    /// | bit 20-22 | 对方飞镖最后一次击中己方前哨站或基地的目标 | 0=默认，1=前哨站，2=基地固定目标，3=基地随机固定目标，4=基地随机移动目标，5=基地末端移动目标 | 开局默认 0 |
    /// | bit 23-24 | 中心增益点占领状态 | 0=未被占领，1=被己方占领，2=被对方占领，3=被双方占领 | 仅 RMUL 适用 |
    /// | bit 25-26 | 己方堡垒增益点占领状态 | 0=未被占领，1=被己方占领，2=被对方占领，3=被双方占领 |  |
    /// | bit 27-28 | 己方前哨站增益点占领状态 | 0=未被占领，1=被己方占领，2=被对方占领 |  |
    /// | bit 29 | 己方基地增益点占领状态 | 1=已占领 |  |
    /// | bit 30-31 | 保留位 | - |  |
    pub const fn event_data(&self) -> u32 {
        self.event_data
    }
}

impl Marshaler for GameEvent {
    const CMD_ID: u16 = 0x0101;
    const PAYLOAD_SIZE: u16 = 4;

    fn marshal(&self, dst: &mut [u8]) -> Result<usize> {
        dst[0..4].copy_from_slice(&self.event_data.to_le_bytes());

        Ok(Self::PAYLOAD_SIZE as usize)
    }

    fn unmarshal(raw: &[u8]) -> Result<Self> {
        let event_data = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);

        Ok(GameEvent { event_data })
    }
}

#[cfg(test)]
#[test]
fn test() {
    const SIZE: usize = GameEvent::PAYLOAD_SIZE as usize;

    let status = GameEvent {
        event_data: 0x12345678,
    };

    let mut buf = [0u8; SIZE + 10];
    let sz = status.marshal(&mut buf).unwrap();
    assert_eq!(sz, SIZE);

    let decoded = GameEvent::unmarshal(&buf[..SIZE]).unwrap();
    assert_eq!(decoded.event_data(), 0x12345678);
}
