use crate::{
    flight::ArmingFlags,
    osd::{
        OsdDrawContext,
        elements::{OsdElement, OsdElements},
        symbols::OsdSymbols,
    },
    sensors::SensorConfig,
};
use core::convert::TryFrom;
use strum::EnumCount;

/*
How to add a new OSD element:

1. Create a new enum, say, `MyElement`, and add it to the `OsdElementId` enumeration list below.
2. Create a drawing function `draw_my_element(&mut self) -> bool`
   and optionally a background drawing function `draw_background_my_element(&mut self) -> bool`.
   for the `OsdElement` `struct`.
3. Add the drawing function to the `draw_element` `match` statement.
4. If you created a background drawing function then add it to the `draw_background_element` `match` statement.
5. Add `OsdElementId::MyElement` to either `OSD_ELEMENT_DISPLAY_ORDER` or (if it is added conditionally at runtime) to
   the active elements in the `add_active_elements` function.
*/

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, EnumCount)]
#[repr(u8)]
pub enum OsdElementId {
    #[default]
    Rssi,
    MainBatteryVoltage,
    Crosshairs,
    ArtificialHorizon,
    HorizonSidebars,
    ItemTimer1,
    ItemTimer2,
    FlyMode,
    CraftName,
    ThrottlePos,
    VtxChannel,
    CurrentDraw,
    MahDrawn,
    GpsSpeed,
    GpsSats,
    Altitude,
    RollPids,
    PitchPids,
    YawPids,
    Power,
    PidRateProfile,
    Warnings,
    AvgCellVoltage,
    GpsLon,
    GpsLat,
    Debug,
    PitchAngle,
    RollAngle,
    MainBatteryUsage,
    Disarmed,
    HomeDirection,
    HomeDistance,
    NumericalHeading,
    NumericalVario,
    CompassBar,
    EscTemperature,
    EscRpm,
    RemainingTimeEstimate,
    RtcDatetime,
    AdjustmentRange,
    CoreTemperature,
    AntiGravity,
    GForce,
    MotorDiagnostics,
    LogStatus,
    FlipArrow,
    LinkQuality,
    FlightDistance,
    StickOverlayLeft,
    StickOverlayRight,
    PilotName,
    EscRpmFrequency,
    RateProfileName,
    PidProfileName,
    ProfileName,
    RssiDbmValue,
    RcChannels,
    CameraFrame,
    Efficiency,
    TotalFlights,
    UpDownReference,
    TxUplinkPower,
    WattHoursDrawn,
    AuxValue,
    ReadyMode,
    RsnrValue,
    SysGoggleVoltage,
    SysVtxVoltage,
    SysBitrate,
    SysDelay,
    SysDistance,
    SysLq,
    SysGoggleDvr,
    SysVtxDvr,
    SysWarnings,
    SysVtxTemperature,
    SysFanSpeed,
    GpsLapTimeCurrent,
    GpsLapTimePrevious,
    GpsLapTimeBest3,
    Debug2,
    CustomMsg0,
    CustomMsg1,
    CustomMsg2,
    CustomMsg3,
    LidarDistance,
    CustomSerialText,
    BatteryProfileName,
}

/// Custom error type for invalid enum index casting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OsdElementIdError;

impl TryFrom<u8> for OsdElementId {
    type Error = OsdElementIdError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        #[allow(clippy::cast_possible_truncation)]
        if value < (OsdElementId::COUNT as u8) {
            // Safe because our enum maps sequentially from 0 up to OSD_ELEMENT_COUNT - 1
            // and contains no custom gaps.
            unsafe { core::mem::transmute::<u8, core::result::Result<OsdElementId, OsdElementIdError>>(value) }
        } else {
            Err(OsdElementIdError)
        }
    }
}

// Convenient conversion helpers for other index sizes commonly found in loop logic
impl TryFrom<usize> for OsdElementId {
    type Error = OsdElementIdError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        #[allow(clippy::cast_possible_truncation)]
        if value < OsdElementId::COUNT {
            // Safe because our enum maps sequentially from 0 up to OSD_ELEMENT_COUNT - 1
            // and contains no custom gaps.
            unsafe { core::mem::transmute::<u8, core::result::Result<OsdElementId, OsdElementIdError>>(value as u8) }
        } else {
            Err(OsdElementIdError)
        }
    }
}

/// Defines the order in which the elements are drawn.
/// Elements positioned later in the list will overlay the earlier ones if their character positions overlap.
/// Elements that need runtime conditional processing should be added directly to `add_active_elements`.
// This compiles down directly to a read-only data block in flash memory.
pub static OSD_ELEMENT_DISPLAY_ORDER: &[OsdElementId] = {
    // Bring OsdElementId into scope to avoid typing OsdElementId:: every time
    #[allow(clippy::enum_glob_use)]
    use OsdElementId::*;
    &[
        ArtificialHorizon,
        GForce,
        UpDownReference,
        MainBatteryVoltage,
        Rssi,
        Crosshairs,
        HorizonSidebars,
        UpDownReference,
        ItemTimer1,
        ItemTimer2,
        RemainingTimeEstimate,
        FlyMode,
        ThrottlePos,
        VtxChannel,
        CurrentDraw,
        MahDrawn,
        WattHoursDrawn,
        CraftName,
        CustomMsg0,
        CustomMsg1,
        CustomMsg2,
        CustomMsg3,
        Altitude,
        RollPids,
        PitchPids,
        YawPids,
        Power,
        PidRateProfile,
        Warnings,
        AvgCellVoltage,
        Debug,
        Debug2,
        PitchAngle,
        RollAngle,
        MainBatteryUsage,
        Disarmed,
        NumericalHeading,
        ReadyMode,
        #[cfg(feature = "barometer")]
        NumericalVario, // Variometer: calculates vertical speed from altitude.
        CompassBar,
        AntiGravity,
        #[cfg(feature = "blackbox")]
        LogStatus,
        MotorDiagnostics,
        FlipArrow,
        PilotName,
        /*#[cfg(feature = "rtc_time")]
        RtcDatetime,

        #[cfg(feature = "osd_adjustments")]
        AdjustmentRange,

        #[cfg(feature = "adc_internal")]
        CoreTemperature,

        #[cfg(feature = "rx_link_quality_info")]
        LinkQuality,

        #[cfg(feature = "rx_link_uplink_power")]
        TxUplinkPower,

        #[cfg(feature = "rx_rssi_dbm")]
        RssiDbmValue,

        #[cfg(feature = "rx_rsnr")]
        RsnrValue,*/
        StickOverlayLeft,
        StickOverlayRight,
        RateProfileName,
        PidProfileName,
        BatteryProfileName,
        ProfileName,
        RcChannels,
        CameraFrame,
        //#[cfg(feature = "use_persistent_stats")]
        //TotalFlights,
        AuxValue,
        #[cfg(feature = "osd_hd")]
        SysGoggleVoltage,
        #[cfg(feature = "osd_hd")]
        SysVtxVoltage,
        #[cfg(feature = "osd_hd")]
        SysBitrate,
        #[cfg(feature = "osd_hd")]
        SysDelay,
        #[cfg(feature = "osd_hd")]
        SysDistance,
        #[cfg(feature = "osd_hd")]
        SysLq,
        #[cfg(feature = "osd_hd")]
        SysGoggleDvr,
        #[cfg(feature = "osd_hd")]
        SysVtxDvr,
        #[cfg(feature = "osd_hd")]
        SysWarnings,
        #[cfg(feature = "osd_hd")]
        SysVtxTemp,
        #[cfg(feature = "osd_hd")]
        SysFanSpeed,
        #[cfg(feature = "rangefinder")]
        LidarDistance,
        //#[cfg(feature = "enable_osd_custom_text")]
        //CustomSerialText,
    ]
};

// element drawing functions
#[allow(unused)]
impl OsdElements {
    pub fn add_active_elements(&mut self, sensors: SensorConfig) {
        for element in OSD_ELEMENT_DISPLAY_ORDER {
            self.add_active_element(*element);
        }

        #[cfg(feature = "gps")]
        if sensors.is_set(SensorConfig::GPS) {
            self.add_active_element(OsdElementId::GpsSats);
            self.add_active_element(OsdElementId::GpsSpeed);
            self.add_active_element(OsdElementId::GpsLat);
            self.add_active_element(OsdElementId::GpsLon);
            self.add_active_element(OsdElementId::HomeDistance);
            self.add_active_element(OsdElementId::HomeDirection);
            self.add_active_element(OsdElementId::FlightDistance);
            self.add_active_element(OsdElementId::Efficiency);
        }
    }

    pub fn draw_element(&mut self, draw_context: &OsdDrawContext) -> bool {
        match self.active_element.id {
            OsdElementId::Rssi => self.active_element.draw_rssi(),
            OsdElementId::MainBatteryVoltage => self.active_element.draw_battery(),
            OsdElementId::ArtificialHorizon => self.active_element.draw_horizon(),
            OsdElementId::PitchAngle => self.active_element.draw_pitch_angle(self.pitch_angle_degrees),
            OsdElementId::RollAngle => self.active_element.draw_roll_angle(self.roll_angle_degrees),
            OsdElementId::Altitude => self.active_element.draw_altitude(),
            OsdElementId::Crosshairs => self.active_element.draw_crosshairs(),
            OsdElementId::NumericalHeading => self.active_element.draw_numerical_heading(),
            OsdElementId::Disarmed => self.active_element.draw_disarmed(draw_context),
            _ => self.active_element.draw_nothing(),
        }
    }
}

#[allow(clippy::unused_self)]
impl OsdElement {
    fn draw_nothing(&self) -> bool {
        false
    }
    fn draw_rssi(&mut self) -> bool {
        true
    }
    fn draw_battery(&mut self) -> bool {
        true
    }
    fn draw_horizon(&mut self) -> bool {
        true
    }
    fn draw_disarmed(&mut self, draw_context: &OsdDrawContext) -> bool {
        if !draw_context.arming_flags.is_set(ArmingFlags::ARMED) {
            self.set_text("DISARMED");
        }
        /*_ = self.write_custom(|w| {
            w.append_str_right_aligned("DISARMED", 8);
        });*/
        true
    }
    fn draw_roll_angle(&mut self, _angle_degrees: f32) -> bool {
        let roll_angle_degrees = 93;
        _ = self.write_custom(|w| {
            w.append_str("ROL:");
            w.append_u32(roll_angle_degrees);
        });
        true
    }
    fn draw_pitch_angle(&mut self, _angle_degrees: f32) -> bool {
        true
    }
    fn draw_numerical_heading(&mut self) -> bool {
        let yaw_angle_degrees = 93;
        _ = self.write_custom(|w| {
            w.append_str("YAW:");
            w.append_u32(yaw_angle_degrees);
        });
        true
    }
    fn draw_altitude(&mut self) -> bool {
        self.buf[0] = OsdSymbols::ALTITUDE;
        self.buf[1] = OsdSymbols::HYPHEN;
        self.buf[2] = 0;
        true
    }
    fn draw_crosshairs(&mut self) -> bool {
        self.buf[0] = OsdSymbols::AH_CENTER_LINE;
        self.buf[1] = OsdSymbols::AH_CENTER;
        self.buf[2] = OsdSymbols::AH_CENTER_LINE_RIGHT;
        self.buf[3] = 0;
        true
    }
}

impl OsdElements {
    pub fn draw_element_background(&mut self, _draw_context: &OsdDrawContext) -> bool {
        match self.active_element.id {
            OsdElementId::HorizonSidebars => self.active_element.draw_background_horizon_sidebars(),
            OsdElementId::CraftName => self.active_element.draw_background_craft_name(),
            OsdElementId::StickOverlayLeft => self.active_element.draw_background_stick_overlay(),
            OsdElementId::PilotName => self.active_element.draw_background_pilot_name(),
            OsdElementId::CameraFrame => self.active_element.draw_background_camera_frame(),
            _ => self.active_element.draw_nothing(),
        }
    }
}

// element background drawing functions
#[allow(clippy::unused_self)]
impl OsdElement {
    fn draw_background_horizon_sidebars(&mut self) -> bool {
        true
    }

    fn draw_background_craft_name(&mut self) -> bool {
        true
    }

    fn draw_background_stick_overlay(&mut self) -> bool {
        true
    }

    fn draw_background_pilot_name(&mut self) -> bool {
        true
    }

    fn draw_background_camera_frame(&mut self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<OsdElementId>();
        is_full::<OsdElementIdError>();
    }
}
