use crate::{
    config::GLOBAL_CONFIG,
    display::{Display, DisplayPortSeverity},
    flight::{ArmingFlags, PidConfig},
    osd::{
        OsdDrawContext,
        elements::{OsdElement, OsdElements, OsdStickCameraFrameRenderPhase, OsdStickOverlayRenderPhase},
        symbols::OsdSymbols,
    },
    tasks::GLOBAL_DEBUG,
};

use core::{convert::TryFrom, fmt::Write};
use radio_controllers::RcMode;
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

// element drawing functions
impl OsdElements {
    #[allow(clippy::too_many_lines)]
    pub async fn draw_element<D: Display>(&mut self, draw_context: &OsdDrawContext<'_, D>) -> bool {
        #[allow(clippy::enum_glob_use)]
        use OsdElementId::*;

        #[allow(clippy::pedantic)]
        match self.active_element.id {
            Rssi => self.active_element.draw_rssi(),
            #[cfg(feature = "battery")]
            MainBatteryVoltage => self.active_element.draw_main_battery_usage(draw_context),
            Crosshairs => self.active_element.draw_crosshairs(),
            ArtificialHorizon => self.active_element.draw_artificial_horizon().await,
            ItemTimer1 | ItemTimer2 => self.active_element.draw_item_timer(),
            FlyMode => self.active_element.draw_fly_mode(),
            ThrottlePos => self.active_element.draw_throttle_position(),
            #[cfg(feature = "vtx")]
            VtxChannel => self.active_element.draw_nothing(),
            #[cfg(feature = "battery")]
            CurrentDraw => self.active_element.draw_current_draw(draw_context),
            #[cfg(feature = "battery")]
            MahDrawn => self.active_element.draw_mah_drawn(draw_context),

            #[cfg(feature = "gps")]
            GpsSpeed => self.active_element.draw_nothing(),
            #[cfg(feature = "gps")]
            GpsSats => self.active_element.draw_nothing(),

            Altitude => self.active_element.draw_altitude(),
            RollPids => self.active_element.draw_roll_pids().await,
            PitchPids => self.active_element.draw_pitch_pids().await,
            YawPids => self.active_element.draw_yaw_pids().await,
            Power => self.active_element.draw_nothing(),
            PidRateProfile => self.active_element.draw_nothing(),
            Warnings => self.active_element.draw_nothing(),
            AvgCellVoltage => self.active_element.draw_nothing(),

            #[cfg(feature = "gps")]
            GpsLon => self.active_element.draw_nothing(),
            #[cfg(feature = "gps")]
            GpsLat => self.active_element.draw_nothing(),

            Debug => self.active_element.draw_debug(),
            PitchAngle => self.active_element.draw_pitch_angle(self.pitch_angle_degrees),
            RollAngle => self.active_element.draw_roll_angle(self.roll_angle_degrees),
            MainBatteryUsage => self.active_element.draw_nothing(),
            Disarmed => self.active_element.draw_disarmed(draw_context),

            #[cfg(feature = "gps")]
            HomeDirection => self.active_element.draw_nothing(),
            #[cfg(feature = "gps")]
            HomeDistance => self.active_element.draw_nothing(),

            NumericalHeading => self.active_element.draw_numerical_heading(self.yaw_angle_degrees),
            NumericalVario => self.active_element.draw_nothing(),
            CompassBar => self.active_element.draw_nothing(),

            #[cfg(feature = "dshot_telemetry")]
            EscTemperature => self.active_element.draw_nothing(),
            #[cfg(feature = "dshot_telemetry")]
            EscRpm => self.active_element.draw_nothing(),

            RemainingTimeEstimate => self.active_element.draw_remaining_time_estimate(),
            RtcDatetime => self.active_element.draw_nothing(),
            AdjustmentRange => self.active_element.draw_nothing(),
            CoreTemperature => self.active_element.draw_nothing(),
            AntiGravity => self.active_element.draw_anti_gravity(draw_context),
            GForce => self.active_element.draw_nothing(),
            MotorDiagnostics => self.active_element.draw_nothing(),

            #[cfg(feature = "blackbox")]
            LogStatus => self.active_element.draw_nothing(),

            FlipArrow => self.active_element.draw_nothing(),
            LinkQuality => self.active_element.draw_nothing(),

            #[cfg(feature = "gps")]
            FlightDistance => self.active_element.draw_nothing(),

            StickOverlayLeft => self.active_element.draw_nothing(),
            StickOverlayRight => self.active_element.draw_nothing(),

            #[cfg(feature = "dshot_telemetry")]
            EscRpmFrequency => self.active_element.draw_nothing(),

            RateProfileName => self.active_element.draw_nothing(),
            PidProfileName => self.active_element.draw_nothing(),
            ProfileName => self.active_element.draw_nothing(),
            RssiDbmValue => self.active_element.draw_nothing(),
            RcChannels => self.active_element.draw_nothing(),

            #[cfg(feature = "gps")]
            Efficiency => self.active_element.draw_nothing(),

            TotalFlights => self.active_element.draw_nothing(),
            UpDownReference => self.active_element.draw_up_down_reference(),
            TxUplinkPower => self.active_element.draw_nothing(),
            WattHoursDrawn => self.active_element.draw_nothing(),
            AuxValue => self.active_element.draw_nothing(),
            ReadyMode => self.active_element.draw_nothing(),
            RsnrValue => self.active_element.draw_nothing(),
            SysGoggleVoltage => self.active_element.draw_nothing(),
            SysVtxVoltage => self.active_element.draw_nothing(),
            SysBitrate => self.active_element.draw_nothing(),
            SysDelay => self.active_element.draw_nothing(),
            SysDistance => self.active_element.draw_nothing(),
            SysLq => self.active_element.draw_nothing(),
            SysGoggleDvr => self.active_element.draw_nothing(),
            SysVtxDvr => self.active_element.draw_nothing(),
            SysWarnings => self.active_element.draw_nothing(),
            SysVtxTemperature => self.active_element.draw_nothing(),
            SysFanSpeed => self.active_element.draw_nothing(),

            #[cfg(feature = "gps")]
            GpsLapTimeCurrent => self.active_element.draw_nothing(),
            #[cfg(feature = "gps")]
            GpsLapTimePrevious => self.active_element.draw_nothing(),
            #[cfg(feature = "gps")]
            GpsLapTimeBest3 => self.active_element.draw_nothing(),
            Debug2 => self.active_element.draw_debug2(),
            CustomMsg0 | CustomMsg1 | CustomMsg2 | CustomMsg3 => self.active_element.draw_custom_message(),
            #[cfg(feature = "rangefinder")]
            LidarDistance => self.active_element.draw_nothing(),
            CustomSerialText => self.active_element.draw_nothing(),
            BatteryProfileName => self.active_element.draw_nothing(),

            // only drawn in background
            CraftName => self.active_element.draw_nothing(), // do nothing, since only drawn in background
            PilotName => self.active_element.draw_nothing(), // do nothing, since only drawn in background
            HorizonSidebars => self.active_element.draw_nothing(), // do nothing, since only drawn in background
            _ => self.active_element.draw_nothing(),
        }
    }

    pub async fn draw_element_background<D: Display>(&mut self, draw_context: &mut OsdDrawContext<'_, D>) -> bool {
        #[allow(clippy::enum_glob_use)]
        use OsdElementId::*;
        match self.active_element.id {
            HorizonSidebars => self.active_element.draw_background_horizon_sidebars(draw_context),
            CraftName => self.active_element.draw_background_craft_name().await,
            StickOverlayLeft => self.active_element.draw_background_stick_overlay(),
            PilotName => self.active_element.draw_background_pilot_name().await,
            CameraFrame => self.active_element.draw_background_camera_frame(draw_context).await,
            _ => self.active_element.draw_nothing(),
        }
    }
}

/// Custom error type for invalid enum index casting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OsdElementIdError;

impl TryFrom<u8> for OsdElementId {
    type Error = OsdElementIdError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if usize::from(value) < OsdElementId::COUNT {
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
        if value < OsdElementId::COUNT {
            // Safe because our enum maps sequentially from 0 up to OSD_ELEMENT_COUNT - 1
            // and contains no custom gaps.
            #[allow(clippy::cast_possible_truncation)]
            unsafe {
                core::mem::transmute::<u8, core::result::Result<OsdElementId, OsdElementIdError>>(value as u8)
            }
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

#[allow(clippy::unused_self)]
impl OsdElement {
    fn draw_nothing(&self) -> bool {
        false
    }

    fn draw_rssi(&mut self) -> bool {
        let rssi = 88;
        _ = write!(self.buf, "{}{:2}", OsdSymbols::RSSI, rssi);
        true
    }

    #[cfg(feature = "battery")]
    fn draw_main_battery_usage<D: Display>(&mut self, draw_context: &OsdDrawContext<D>) -> bool {
        const USAGE_STEPS: usize = 11; // Use an odd number so the bar can be centered.

        _ = draw_context.battery_message;

        // TODO: calculate battery bars from the battery data
        //let remaining_capacity_bars = 4;
        // Setup the boundaries
        self.buf[0] = OsdSymbols::PB_START;
        self.buf[USAGE_STEPS + 1] = OsdSymbols::PB_CLOSE;

        // Fill the battery bar using an iterator slice
        /*let range = 1..=USAGE_STEPS;
        for (ii, symbol) in self.buf[range].iter_mut().enumerate() {
            *symbol = if ii < remaining_capacity_bars { OsdSymbols::PB_FULL } else { OsdSymbols::PB_EMPTY };
        }

        // Handle the end-cap symbol if needed
        if (1..USAGE_STEPS).contains(&remaining_capacity_bars) {
            self.buf[1 + remaining_capacity_bars] = OsdSymbols::PB_END;
        }*/
        true
    }

    fn draw_crosshairs(&mut self) -> bool {
        self.buf[0] = OsdSymbols::AH_CENTER_LINE;
        self.buf[1] = OsdSymbols::AH_CENTER;
        self.buf[2] = OsdSymbols::AH_CENTER_LINE_RIGHT;
        self.buf[3] = 0;
        true
    }

    async fn draw_artificial_horizon(&mut self) -> bool {
        const AH_SYMBOL_COUNT: i32 = 9;
        let osd_config = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.osd
        };
        // Get pitch and roll limits in tenths of degrees
        let max_pitch = i32::from(osd_config.ah_max_pitch * 10);
        let max_roll = i32::from(osd_config.ah_max_roll * 10);
        let ah_sign = if osd_config.ah_invert == 0 { 1 } else { -1 };
        let roll = 0;
        let pitch = 0;
        let roll_angle = (roll * ah_sign).clamp(-max_roll, max_roll);
        let mut pitch_angle = (pitch * ah_sign).clamp(-max_pitch, max_pitch);
        // Convert pitchAngle to y compensation value
        // (max_pitch / 25) divisor matches previous settings of fixed divisor of 8 and fixed max AHI pitch angle of 20.0 degrees
        if max_pitch > 0 {
            pitch_angle = (pitch_angle * 25) / max_pitch;
        }
        pitch_angle -= 4 * AH_SYMBOL_COUNT + 5;

        let y: i32 = (-roll_angle * self.horizon_x) / 64 - pitch_angle;
        #[allow(clippy::cast_possible_truncation)]
        if (0..=81).contains(&y) {
            self.offset_x = self.horizon_x.cast_unsigned() as u8;
            self.offset_y = (y / AH_SYMBOL_COUNT).cast_unsigned() as u8;

            self.buf[0] = OsdSymbols::AH_BAR9_0 + (y % AH_SYMBOL_COUNT).cast_unsigned() as u8;
            self.draw_element = true;
        } else {
            self.draw_element = false; // element does not need to be rendered
        }

        if self.horizon_x == 4 {
            // Rendering is complete, so prepare to start again
            self.horizon_x = -4;
        } else {
            // Rendering not yet complete
            self.rendered = false;
            self.horizon_x += 1;
        }
        self.draw_element
    }

    fn draw_item_timer(&mut self) -> bool {
        true
    }

    fn draw_fly_mode(&mut self) -> bool {
        true
    }

    fn draw_throttle_position(&mut self) -> bool {
        true
    }

    #[cfg(feature = "battery")]
    fn draw_current_draw<D: Display>(&mut self, draw_context: &OsdDrawContext<D>) -> bool {
        let amperage = draw_context.battery_message.current.amperage_x100;
        _ = write!(self.buf, "{:3}{}", amperage, OsdSymbols::AMP);
        true
    }

    #[cfg(feature = "battery")]
    fn draw_mah_drawn<D: Display>(&mut self, draw_context: &OsdDrawContext<D>) -> bool {
        let mah_drawn = draw_context.battery_message.current.mah_drawn;
        if mah_drawn >= self.osd_cap_alarm.into() {
            self.attr = DisplayPortSeverity::Normal;
        }
        _ = write!(self.buf, "{:4}{}", mah_drawn, OsdSymbols::MAH);
        true
    }

    fn draw_altitude(&mut self) -> bool {
        self.buf[0] = OsdSymbols::ALTITUDE;
        self.buf[1] = OsdSymbols::HYPHEN;
        self.buf[2] = 0;
        true
    }

    pub fn format_pid(&mut self, label: &str, pid: PidConfig) {
        _ = write!(self.buf, "{} {:3} {:3} {:3} {:3} {:3}", label, pid.kp, pid.ki, pid.kd, pid.ks, pid.kk);
    }

    async fn draw_roll_pids(&mut self) -> bool {
        let pid_config = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.pid_roll_rate
        };
        self.format_pid("ROL", pid_config);
        true
    }

    async fn draw_pitch_pids(&mut self) -> bool {
        let pid_config = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.pid_pitch_rate
        };
        self.format_pid("PIT", pid_config);
        true
    }

    async fn draw_yaw_pids(&mut self) -> bool {
        let pid_config = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.pid_roll_rate
        };
        self.format_pid("YAW", pid_config);
        true
    }

    fn draw_debug(&mut self) -> bool {
        let debug = GLOBAL_DEBUG.values();
        _ = write!(self.buf, "DBG {:5} {:5} {:5} {:5}", debug[0], debug[1], debug[2], debug[3]);
        true
    }

    fn draw_debug2(&mut self) -> bool {
        let debug = GLOBAL_DEBUG.values();
        _ = write!(self.buf, "DBG {:5} {:5} {:5} {:5}", debug[4], debug[5], debug[6], debug[7]);
        true
    }

    fn draw_pitch_angle(&mut self, angle_degrees: i32) -> bool {
        let sign_char = if angle_degrees < 0 { '-' } else { ' ' };
        let angle_abs = angle_degrees.unsigned_abs(); // Converts to unsigned, avoiding negation overflow
        _ = write!(self.buf, "{}{}{:02}", OsdSymbols::ROLL, sign_char, angle_abs);
        true
    }

    fn draw_roll_angle(&mut self, angle_degrees: i32) -> bool {
        // floor is supported natively on ARM Cortex-M, round is not
        let sign_char = if angle_degrees < 0 { '-' } else { ' ' };
        let angle_abs = angle_degrees.unsigned_abs(); // Converts to unsigned, avoiding negation overflow
        _ = write!(self.buf, "{}{}{:02}", OsdSymbols::ROLL, sign_char, angle_abs);
        true
    }

    fn draw_disarmed<D: Display>(&mut self, draw_context: &OsdDrawContext<D>) -> bool {
        if !draw_context.arming_flags.is_set(ArmingFlags::ARMED) {
            self.write_string("DISARMED");
        }
        /*_ = self.write_custom(|w| {
            w.append_str_right_aligned("DISARMED", 8);
        });*/
        true
    }

    fn direction_symbol_from_heading(heading_degrees: i32) -> u8 {
        let heading_degrees = heading_degrees + 360; // Ensure positive value

        // Split input heading 0..359 into sectors 0..(directions-1), but offset
        // by half a sector so that sector 0 gets centered around heading 0.
        // We multiply heading by directions to not loose precision in divisions
        // In this way each segment will be a FULL_CIRCLE length
        let mut direction = (heading_degrees * 16 + 180) / 360; // scale with rounding
        direction %= 16;
        #[allow(clippy::cast_possible_truncation)]
        let mut heading = direction.cast_unsigned() as u8;

        // Now heading has a heading with Up=0, Right=4, Down=8 and Left=12
        // Our symbols are Down=0, Right=4, Up=8 and Left=12
        // There're 16 arrow symbols. Transform it.
        heading = 16 - heading;
        heading = (heading + 8) % 16;

        OsdSymbols::ARROW_SOUTH + heading
    }

    fn draw_numerical_heading(&mut self, angle_degrees: i32) -> bool {
        _ = write!(self.buf, "{}{:03}", Self::direction_symbol_from_heading(angle_degrees), angle_degrees);
        true
    }

    fn draw_remaining_time_estimate(&mut self) -> bool {
        true
    }

    fn draw_anti_gravity<D: Display>(&mut self, draw_context: &OsdDrawContext<D>) -> bool {
        if draw_context.active_modes.test(RcMode::ANTIGRAVITY) {
            self.write_string("AG");
        }
        true
    }

    fn draw_up_down_reference(&mut self) -> bool {
        true
    }

    fn draw_custom_message(&mut self) -> bool {
        true
    }
}

// element background drawing functions
impl OsdElement {
    fn draw_background_horizon_sidebars<D: Display>(&mut self, draw_context: &mut OsdDrawContext<D>) -> bool {
        const AH_SIDEBAR_WIDTH_POS: u8 = 7;
        const AH_SIDEBAR_HEIGHT_POS: i8 = 3;

        self.sidebar_render_level = false;
        self.sidebar_y = -AH_SIDEBAR_HEIGHT_POS;
        // Draw AH sides
        let hud_width = AH_SIDEBAR_WIDTH_POS;
        let hud_height = AH_SIDEBAR_HEIGHT_POS;

        if self.sidebar_render_level {
            // AH level indicators
            _ = draw_context.display_port.write_char(
                self.pos_x - hud_width + 1,
                self.pos_y,
                OsdSymbols::AH_LEFT,
                DisplayPortSeverity::Normal,
            );
            _ = draw_context.display_port.write_char(
                self.pos_x + hud_width - 1,
                self.pos_y,
                OsdSymbols::AH_RIGHT,
                DisplayPortSeverity::Normal,
            );
            self.sidebar_render_level = false;
        } else {
            _ = draw_context.display_port.write_char(
                self.pos_x - hud_width,
                (self.pos_y.cast_signed() + self.sidebar_y).cast_unsigned(),
                OsdSymbols::AH_DECORATION,
                DisplayPortSeverity::Normal,
            );
            _ = draw_context.display_port.write_char(
                self.pos_x + hud_width,
                (self.pos_y.cast_signed() + self.sidebar_y).cast_unsigned(),
                OsdSymbols::AH_DECORATION,
                DisplayPortSeverity::Normal,
            );

            if self.sidebar_y == hud_height {
                // Rendering is complete, so prepare to start again
                self.sidebar_y = -hud_height;
                // On next pass render the level markers
                self.sidebar_render_level = true;
            } else {
                self.sidebar_y += 1;
            }
            // Rendering not yet complete
            self.rendered = false;
        }

        self.draw_element = false; // element already drawn
        true
    }

    async fn draw_background_craft_name(&mut self) -> bool {
        let pilot_config = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.pilot
        };
        if pilot_config.craft_name.length == 0 {
            self.write_string("CRAFT_NAME");
        } else {
            self.write_slice(pilot_config.craft_name.as_bytes());
        }
        true
    }

    fn draw_background_stick_overlay(&mut self) -> bool {
        const OSD_STICK_OVERLAY_WIDTH: usize = 7;
        const OSD_STICK_OVERLAY_HEIGHT: u8 = 5;

        if self.stick_overlay_render_phase == OsdStickOverlayRenderPhase::Vertical {
            self.buf[0] = OsdSymbols::STICK_OVERLAY_VERTICAL;
            self.offset_y = self.stick_overlay_y;
            self.stick_overlay_y += 1;
            if self.stick_overlay_y == (OSD_STICK_OVERLAY_HEIGHT - 1) / 2 {
                // Skip over horizontal
                self.stick_overlay_y += 1;
            }
            if self.stick_overlay_y == OSD_STICK_OVERLAY_HEIGHT {
                self.stick_overlay_y = 0;
                self.stick_overlay_render_phase = OsdStickOverlayRenderPhase::Horizontal;
            }
            self.rendered = false;
        } else {
            self.buf.buf[..OSD_STICK_OVERLAY_WIDTH].fill(OsdSymbols::STICK_OVERLAY_HORIZONTAL);
            self.buf[(OSD_STICK_OVERLAY_WIDTH - 1) / 2] = OsdSymbols::STICK_OVERLAY_CENTER;
            self.buf[OSD_STICK_OVERLAY_WIDTH] = 0; // string terminator

            self.offset_y = (OSD_STICK_OVERLAY_HEIGHT - 1) / 2;
            self.stick_overlay_render_phase = OsdStickOverlayRenderPhase::Vertical;
        }
        true
    }

    async fn draw_background_pilot_name(&mut self) -> bool {
        let pilot_config = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.pilot
        };
        if pilot_config.craft_name.length == 0 {
            self.write_string("PILOT_NAME");
        } else {
            self.write_slice(pilot_config.pilot_name.as_bytes());
        }
        true
    }

    async fn draw_background_camera_frame<D: Display>(&mut self, draw_context: &mut OsdDrawContext<'_, D>) -> bool {
        const OSD_CAMERA_FRAME_MIN_WIDTH: u8 = 2;
        const OSD_CAMERA_FRAME_MAX_WIDTH: u8 = 30; // Characters per row supported by MAX7456
        const OSD_CAMERA_FRAME_MIN_HEIGHT: u8 = 2;
        const OSD_CAMERA_FRAME_MAX_HEIGHT: u8 = 16; // Rows supported by MAX7456 (PAL)

        let xpos = self.pos_x;
        let ypos = self.pos_y;
        let osd_config = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.osd
        };
        let width = osd_config.camera_frame_width.clamp(OSD_CAMERA_FRAME_MIN_WIDTH, OSD_CAMERA_FRAME_MAX_WIDTH);
        let height = osd_config.camera_frame_height.clamp(OSD_CAMERA_FRAME_MIN_HEIGHT, OSD_CAMERA_FRAME_MAX_HEIGHT);

        if self.camera_frame_render_phase != OsdStickCameraFrameRenderPhase::Bottom {
            // Rendering not yet complete
            self.rendered = false;
        }

        if self.camera_frame_render_phase == OsdStickCameraFrameRenderPhase::Middle {
            self.camera_frame_i = 1;

            _ = draw_context.display_port.write_char(
                xpos,
                ypos + self.camera_frame_i,
                OsdSymbols::STICK_OVERLAY_VERTICAL,
                DisplayPortSeverity::Normal,
            );
            _ = draw_context.display_port.write_char(
                xpos + width - 1,
                ypos + self.camera_frame_i,
                OsdSymbols::STICK_OVERLAY_VERTICAL,
                DisplayPortSeverity::Normal,
            );

            self.draw_element = false; // element already drawn

            self.camera_frame_i += 1;
            if self.camera_frame_i == height {
                self.camera_frame_i = 1;
                self.camera_frame_render_phase = OsdStickCameraFrameRenderPhase::Bottom;
            }
        } else {
            self.buf[0] = OsdSymbols::STICK_OVERLAY_CENTER;
            self.buf[1..(width as usize - 1)].fill(OsdSymbols::STICK_OVERLAY_HORIZONTAL);
            self.buf[width as usize - 1] = OsdSymbols::STICK_OVERLAY_CENTER;
            self.buf[width as usize] = 0; // string terminator

            if self.camera_frame_render_phase == OsdStickCameraFrameRenderPhase::Top {
                self.camera_frame_render_phase = OsdStickCameraFrameRenderPhase::Middle;
            } else {
                self.offset_y = height - 1;
                self.camera_frame_render_phase = OsdStickCameraFrameRenderPhase::Top;
            }
        }
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
