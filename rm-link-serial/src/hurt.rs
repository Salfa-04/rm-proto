use crate::private::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Reason {
    /// 装甲模块被弹丸击中导致扣血
    HitByProjectile = 0,
    /// 装甲模块或超级电容管理模块离线导致扣血
    ModuleOffline = 1,
    /// 装甲模块受到撞击导致扣血
    StruckByImpact = 5,
}

/// 伤害状态数据，伤害发生后发送
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HurtData {
    armor_id: u8,
    deduction_reason: Reason,
}

impl HurtData {
    pub const fn new(armor_id: u8, deduction_reason: Reason) -> HurtData {
        Self {
            armor_id,
            deduction_reason,
        }
    }
}

impl HurtData {
    /// 当扣血原因为装甲模块被弹丸攻击、受撞击或离线时，
    /// 该数值为装甲模块或测速模块的 ID 编号；
    /// 当其他原因导致扣血时，该数值为 0。
    pub const fn armor_id(&self) -> u8 {
        self.armor_id
    }

    /// 血量变化类型
    ///
    /// | 值 | 说明 |
    /// |---:|---|
    /// | 0 | 装甲模块被弹丸攻击导致扣血 |
    /// | 1 | 装甲模块或超级电容管理模块离线导致扣血 |
    /// | 5 | 装甲模块受到撞击导致扣血 |
    pub const fn deduction_reason(&self) -> Reason {
        self.deduction_reason
    }
}

impl Marshaler for HurtData {
    const CMD_ID: u16 = 0x0206;
    const PAYLOAD_SIZE: u16 = 1;

    fn marshal(&self, dst: &mut [u8]) -> Result<usize> {
        dst[0] = (self.armor_id & 0xF) | (((self.deduction_reason as u8) & 0xF) << 4);

        Ok(Self::PAYLOAD_SIZE as usize)
    }

    fn unmarshal(raw: &[u8]) -> Result<Self> {
        let armor_id = raw[0] & 0x0F;
        let deduction_reason = match (raw[0] >> 4) & 0xF {
            0 => Reason::HitByProjectile,
            1 => Reason::ModuleOffline,
            5 => Reason::StruckByImpact,

            _ => {
                return Err((0, "Invalid Deduction Reason").into());
            }
        };

        Ok(HurtData {
            armor_id,
            deduction_reason,
        })
    }
}

#[cfg(test)]
#[test]
fn test() {
    const SIZE: usize = HurtData::PAYLOAD_SIZE as usize;

    let hurt = HurtData {
        armor_id: 3,
        deduction_reason: Reason::ModuleOffline,
    };

    let mut buf = [0u8; SIZE];
    let sz = hurt.marshal(&mut buf).unwrap();
    assert_eq!(sz, SIZE);
    assert_eq!(buf[0], 0x13);

    let hurt2 = HurtData::unmarshal(&buf).unwrap();
    assert_eq!(hurt2.armor_id(), 3);
    assert_eq!(hurt2.deduction_reason(), Reason::ModuleOffline);
}
