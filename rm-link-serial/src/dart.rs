use crate::private::*;

/// 飞镖发射相关数据，固定以 1Hz 频率发送
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DartInfo {
    remaining_time: u8,
    dart_info: u16,
}

impl DartInfo {
    pub const fn new(remaining_time: u8, dart_info: u16) -> DartInfo {
        Self {
            remaining_time,
            dart_info,
        }
    }
}

impl DartInfo {
    /// 己方飞镖发射剩余时间，单位：秒
    pub const fn remaining_time(&self) -> u8 {
        self.remaining_time
    }

    /// 飞镖信息，包含以下内容：
    ///
    /// | Bits  | Description |
    /// |-------|-------------|
    /// | 0-2   | 最近一次己方飞镖击中的目标：0=开局默认, 1=击中前哨站, 2=击中基地固定目标, 3=击中基地随机固定目标, 4=击中基地随机移动目标, 5=击中基地末端移动目标 |
    /// | 3-5   | 对方最近被击中的目标累计被击中计次数：开局默认为0, 至多为4 |
    /// | 6-8   | 飞镖此时选定的击打目标：0=开局默认/未选定/选定前哨站, 1=选中基地固定目标, 2=选中基地随机固定目标, 3=选中基地随机移动目标, 4=选中基地末端移动目标 |
    /// | 9-15  | 保留位 |
    pub const fn dart_info(&self) -> u16 {
        self.dart_info
    }
}

impl Marshaler for DartInfo {
    const CMD_ID: u16 = 0x0105;
    const PAYLOAD_SIZE: u16 = 3;

    fn marshal(&self, dst: &mut [u8]) -> Result<usize> {
        dst[0] = self.remaining_time;
        dst[1..3].copy_from_slice(&self.dart_info.to_le_bytes());

        Ok(Self::PAYLOAD_SIZE as usize)
    }

    fn unmarshal(raw: &[u8]) -> Result<Self> {
        let remaining_time = raw[0];
        let dart_info = u16::from_le_bytes([raw[1], raw[2]]);

        Ok(DartInfo {
            remaining_time,
            dart_info,
        })
    }
}

#[cfg(test)]
#[test]
fn test() {
    const SIZE: usize = DartInfo::PAYLOAD_SIZE as usize;

    let status = DartInfo {
        remaining_time: 120,
        dart_info: 0x3456,
    };

    let mut buf = [0u8; SIZE + 10];
    let sz = status.marshal(&mut buf).unwrap();
    assert_eq!(sz, SIZE);

    let decoded = DartInfo::unmarshal(&buf[..SIZE]).unwrap();
    assert_eq!(decoded.remaining_time(), 120);
    assert_eq!(decoded.dart_info(), 0x3456);
}
