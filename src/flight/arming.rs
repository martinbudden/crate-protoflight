#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ArmingConfig {
    pub gyro_cal_on_first_arm: u8, // calibrate the gyro right before the first arm
    pub auto_disarm_delay: u8, // allow automatically disarming multicopters after auto_disarm_delay seconds of zero throttle. Disabled when 0
    pub prearm_allow_rearm: u8,
}

impl ArmingConfig {
    pub const fn new() -> Self {
        Self { gyro_cal_on_first_arm: 0, auto_disarm_delay: 5, prearm_allow_rearm: 0 }
    }
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for ArmingConfig {}

impl Default for ArmingConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ArmingFlags {
    pub flags: u8,
}

#[allow(unused)]
impl ArmingFlags {
    pub const ARMED: u8 = (1 << 0);
    pub const WAS_EVER_ARMED: u8 = (1 << 1);
    pub const WAS_ARMED_WITH_PREARM: u8 = (1 << 2);

    pub const fn new() -> Self {
        Self { flags: 0 }
    }

    pub fn set(&mut self, flag: u8) {
        self.flags |= flag;
    }

    pub fn is_set(self, flag: u8) -> bool {
        self.flags & flag != 0
    }

    pub fn flags(self) -> u8 {
        self.flags
    }
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for ArmingFlags {}

impl Default for ArmingFlags {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DisarmingFlags {
    pub flags: u32,
}

#[allow(unused)]
impl DisarmingFlags {
    pub const NO_GYRO: u32 = 1 << 0;
    pub const FAILSAFE: u32 = 1 << 1;
    pub const RX_FAILSAFE: u32 = 1 << 2;
    pub const NOT_DISARMED: u32 = 1 << 3;
    pub const BOX_FAILSAFE: u32 = 1 << 4;
    pub const RUNAWAY_TAKEOFF: u32 = 1 << 5;
    pub const CRASH_DETECTED: u32 = 1 << 6;
    pub const THROTTLE: u32 = 1 << 7;
    pub const ANGLE: u32 = 1 << 8;
    pub const BOOT_GRACE_TIME: u32 = 1 << 9;
    pub const NO_PREARM: u32 = 1 << 10;
    pub const LOAD: u32 = 1 << 11;
    pub const CALIBRATING: u32 = 1 << 12;
    pub const CLI: u32 = 1 << 13;
    pub const CMS_MENU: u32 = 1 << 14;
    pub const BST: u32 = 1 << 15;
    pub const MSP: u32 = 1 << 16;
    pub const PARALYZE: u32 = 1 << 17;
    pub const GPS: u32 = 1 << 18;
    pub const R_ESC: u32 = 1 << 19;
    pub const DSHOT_TELEMETRY: u32 = 1 << 20;
    pub const REBOOT_REQUIRED: u32 = 1 << 21;
    pub const DSHOT_BITBANG: u32 = 1 << 22;
    pub const ACC_CALIBRATION: u32 = 1 << 23;
    pub const MOTOR_PROTOCOL: u32 = 1 << 24;
    pub const CRASH_FLIP: u32 = 1 << 25;
    pub const ALT_HOLD: u32 = 1 << 26;
    pub const POS_HOLD: u32 = 1 << 27;
    pub const ARM_SWITCH: u32 = 1 << 28; // Needs to be the last element, since it's always activated if one of the others is active when arming

    pub const fn new() -> Self {
        Self { flags: 0 }
    }

    pub fn set(&mut self, flag: u32) {
        self.flags |= flag;
    }

    pub fn is_set(self, flag: u32) -> bool {
        self.flags & flag != 0
    }

    pub fn flags(self) -> u32 {
        self.flags
    }
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for DisarmingFlags {}

impl Default for DisarmingFlags {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<ArmingFlags>();
        is_full::<DisarmingFlags>();

        #[cfg(feature = "serde")]
        is_config::<ArmingFlags>();
        #[cfg(feature = "serde")]
        is_config::<DisarmingFlags>();
    }
}
