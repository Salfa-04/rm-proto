use crate::private::*;

/// 自定义控制器与机器人交互数据，发送方触发发送，频率上限为 30Hz
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Custom2Robot {
    // 自定义数据： 6 个关节角度 + 夹爪开合状态
    joints: [f32; 6], // 24 Bytes
    gripper: bool,    // 1 Byte
}

impl Custom2Robot {
    pub const fn get_joints(&self) -> &[f32; 6] {
        &self.joints
    }

    pub const fn get_gripper(&self) -> bool {
        self.gripper
    }
}

impl ImplCommandMsg for Custom2Robot {
    const CMD_ID: u16 = 0x0302;
    const PAYLOAD_SIZE: u16 = 30;
}

impl ImplUnMarshal for Custom2Robot {
    fn unmarshal(raw: &[u8]) -> Result<Self> {
        let joints = [
            f32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]),
            f32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]),
            f32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]),
            f32::from_le_bytes([raw[12], raw[13], raw[14], raw[15]]),
            f32::from_le_bytes([raw[16], raw[17], raw[18], raw[19]]),
            f32::from_le_bytes([raw[20], raw[21], raw[22], raw[23]]),
        ];

        let gripper = raw[24] != 0;

        Ok(Custom2Robot { joints, gripper })
    }
}
