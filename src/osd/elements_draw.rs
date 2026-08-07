#![cfg(feature = "osd")]

use crate::{
    config::GLOBAL_CONFIG,
    display::{Display, DisplayPortSeverity},
    flight::{ArmingFlags, PidConfig},
    osd::{
        Osd, OsdConfig, OsdDrawContext,
        elements::{
            OsdElement, OsdElements, OsdElementsCache, OsdStickCameraFrameRenderPhase, OsdStickOverlayRenderPhase,
        },
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
    pub async fn draw_element_foreground(
        draw_context: &OsdDrawContext,
        osd_element: &mut OsdElement,
        osd_config: &OsdConfig,
        cache: OsdElementsCache,
    ) -> (bool, bool) {
        #[allow(clippy::enum_glob_use)]
        use OsdElementId::*;

        #[allow(clippy::pedantic)]
        let drawn = match osd_element.id {
            Rssi => osd_element.draw_rssi(),
            #[cfg(feature = "battery")]
            MainBatteryVoltage => osd_element.draw_main_battery_usage(),
            Crosshairs => osd_element.draw_crosshairs(),
            ArtificialHorizon => osd_element.draw_artificial_horizon(osd_config),
            ItemTimer1 | ItemTimer2 => osd_element.draw_item_timer(),
            FlyMode => osd_element.draw_fly_mode(),
            ThrottlePos => osd_element.draw_throttle_position(),
            #[cfg(feature = "vtx")]
            VtxChannel => osd_element.draw_vtx_channel(),
            #[cfg(feature = "battery")]
            CurrentDraw => osd_element.draw_current_draw(draw_context),
            #[cfg(feature = "battery")]
            MahDrawn => osd_element.draw_mah_drawn(draw_context),

            #[cfg(feature = "gps")]
            GpsSpeed => osd_element.draw_gps_speed(),
            #[cfg(feature = "gps")]
            GpsSats => osd_element.draw_gps_sats(),

            Altitude => osd_element.draw_altitude(),
            RollPids => osd_element.draw_roll_pids().await,
            PitchPids => osd_element.draw_pitch_pids().await,
            YawPids => osd_element.draw_yaw_pids().await,
            Power => osd_element.draw_power(),
            PidRateProfile => osd_element.draw_pid_rate_profile(),
            Warnings => osd_element.draw_warnings(),
            AvgCellVoltage => osd_element.draw_average_cell_voltage(),

            #[cfg(feature = "gps")]
            GpsLon => osd_element.draw_gps_lon(),
            #[cfg(feature = "gps")]
            GpsLat => osd_element.draw_gps_lat(),

            Debug => osd_element.draw_debug(),
            PitchAngle => osd_element.draw_pitch_angle(cache.pitch_angle_degrees),
            RollAngle => osd_element.draw_roll_angle(cache.roll_angle_degrees),
            MainBatteryUsage => osd_element.draw_nothing(),
            Disarmed => osd_element.draw_disarmed(draw_context),

            #[cfg(feature = "gps")]
            HomeDirection => osd_element.draw_home_direction(),
            #[cfg(feature = "gps")]
            HomeDistance => osd_element.draw_home_distance(),

            NumericalHeading => osd_element.draw_numerical_heading(cache.yaw_angle_degrees),
            NumericalVario => osd_element.draw_numerical_vario(),
            CompassBar => osd_element.draw_compass_bar(),

            #[cfg(feature = "dshot_telemetry")]
            EscTemperature => osd_element.draw_esc_temperature(),
            #[cfg(feature = "dshot_telemetry")]
            EscRpm => osd_element.draw_esc_rpm(),

            RemainingTimeEstimate => osd_element.draw_remaining_time_estimate(),
            RtcDatetime => osd_element.draw_rtc_date_time(),
            AdjustmentRange => osd_element.draw_adjustment_range(),
            CoreTemperature => osd_element.draw_core_temperature(),
            AntiGravity => osd_element.draw_anti_gravity(draw_context),
            GForce => osd_element.draw_g_force(),
            MotorDiagnostics => osd_element.draw_motor_diagnostics(),

            #[cfg(feature = "blackbox")]
            LogStatus => osd_element.draw_log_status(),

            FlipArrow => osd_element.draw_flip_arrow(),
            LinkQuality => osd_element.draw_link_quality(),

            #[cfg(feature = "gps")]
            FlightDistance => osd_element.draw_flight_distance(),

            StickOverlayLeft | StickOverlayRight => osd_element.draw_stick_overlay(),

            #[cfg(feature = "dshot_telemetry")]
            EscRpmFrequency => osd_element.draw_esc_rpm_frequency(),

            RateProfileName => osd_element.draw_rate_profile_name(),
            PidProfileName => osd_element.draw_pid_profile_name(),
            ProfileName => osd_element.draw_profile_name(),
            RssiDbmValue => osd_element.draw_rssi_dmb_value(),
            RcChannels => {
                osd_element.draw_rc_channels(draw_context.rx_message.rc_controls.controls_pwm, osd_config.rc_channels)
            }

            #[cfg(feature = "gps")]
            Efficiency => osd_element.draw_efficiency(),

            TotalFlights => osd_element.draw_total_flights(),
            UpDownReference => osd_element.draw_up_down_reference(),
            TxUplinkPower => osd_element.draw_tx_uplink_power(),
            WattHoursDrawn => osd_element.draw_watt_hours_drawn(),
            AuxValue => osd_element.draw_aux_value(),
            ReadyMode => osd_element.draw_ready_mode(),
            RsnrValue => osd_element.draw_rsnr_value(),
            /*SysGoggleVoltage => osd_element.draw_nothing(),
            SysVtxVoltage => osd_element.draw_nothing(),
            SysBitrate => osd_element.draw_nothing(),
            SysDelay => osd_element.draw_nothing(),
            SysDistance => osd_element.draw_nothing(),
            SysLq => osd_element.draw_nothing(),
            SysGoggleDvr => osd_element.draw_nothing(),
            SysVtxDvr => osd_element.draw_nothing(),
            SysWarnings => osd_element.draw_nothing(),
            SysVtxTemperature => osd_element.draw_nothing(),
            SysFanSpeed => osd_element.draw_nothing(),*/
            #[cfg(feature = "gps")]
            GpsLapTimeCurrent => osd_element.draw_lap_time_current(),
            #[cfg(feature = "gps")]
            GpsLapTimePrevious => osd_element.draw_lap_time_previous(),
            #[cfg(feature = "gps")]
            GpsLapTimeBest3 => osd_element.draw_lap_time_best3(),
            Debug2 => osd_element.draw_debug2(),
            CustomMsg0 | CustomMsg1 | CustomMsg2 | CustomMsg3 => osd_element.draw_custom_message(),
            #[cfg(feature = "rangefinder")]
            LidarDistance => osd_element.draw_lidar_distance(),
            CustomSerialText => osd_element.draw_custom_serial_text(),
            BatteryProfileName => osd_element.draw_battery_profile_name(),

            // only drawn in background
            CraftName | PilotName | HorizonSidebars => osd_element.draw_nothing(), // do nothing, since only drawn in background
            _ => osd_element.draw_nothing(),
        };
        (drawn, osd_element.rendered)
    }

    pub async fn draw_element_background<D: Display>(
        display_port: &mut D,
        osd_element: &mut OsdElement,
        osd_config: &OsdConfig,
    ) -> (bool, bool) {
        #[allow(clippy::enum_glob_use)]
        use OsdElementId::*;
        let drawn = match osd_element.id {
            HorizonSidebars => osd_element.draw_background_horizon_sidebars(display_port), // Background only
            CraftName => osd_element.draw_background_craft_name().await,                   // Background only
            StickOverlayLeft | StickOverlayRight => osd_element.draw_background_stick_overlay(), // Background and foreground
            PilotName => osd_element.draw_background_pilot_name().await,                         // Background only
            CameraFrame => osd_element.draw_background_camera_frame(display_port, osd_config),   // Background only
            _ => osd_element.draw_nothing(),
        };
        (drawn, osd_element.rendered)
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
        /*
        RtcDatetime,
        AdjustmentRange,
        CoreTemperature,
        LinkQuality,
        TxUplinkPower,
        RssiDbmValue,
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
/// Draw functions return false if the element is not fully rendered and requires more draw steps to complete the drawing.
/// Used by complex elements like `draw_artificial_horizon` that require multi-step drawing.
impl OsdElement {
    fn draw_nothing(&self) -> bool {
        false
    }

    fn draw_rssi(&mut self) -> bool {
        let rssi = 88;
        _ = write!(self.fixed_buf, "{}{:2}", OsdSymbols::RSSI, rssi);
        true
    }

    #[cfg(feature = "battery")]
    fn draw_main_battery_usage(&mut self) -> bool {
        const USAGE_STEPS: usize = 11; // Use an odd number so the bar can be centered.

        //_ = draw_context.battery_message;

        // TODO: calculate battery bars from the battery data
        //let remaining_capacity_bars = 4;
        // Setup the boundaries
        self.fixed_buf[0] = OsdSymbols::PB_START;
        self.fixed_buf[USAGE_STEPS + 1] = OsdSymbols::PB_CLOSE;

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
        self.fixed_buf[0] = OsdSymbols::AH_CENTER_LINE;
        self.fixed_buf[1] = OsdSymbols::AH_CENTER;
        self.fixed_buf[2] = OsdSymbols::AH_CENTER_LINE_RIGHT;
        self.fixed_buf[3] = 0;
        true
    }

    fn draw_artificial_horizon(&mut self, osd_config: &OsdConfig) -> bool {
        const AH_SYMBOL_COUNT: i32 = 9;
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

        let y: i32 = (-roll_angle * self.state.horizon_x) / 64 - pitch_angle;
        #[allow(clippy::cast_possible_truncation)]
        if (0..=81).contains(&y) {
            self.offset_x = self.state.horizon_x.cast_unsigned() as u8;
            self.offset_y = (y / AH_SYMBOL_COUNT).cast_unsigned() as u8;

            self.fixed_buf[0] = OsdSymbols::AH_BAR9_0 + (y % AH_SYMBOL_COUNT).cast_unsigned() as u8;
            self.draw_element = true;
        } else {
            self.draw_element = false; // element does not need to be rendered
        }

        if self.state.horizon_x == 4 {
            // Rendering is complete, so prepare to start again
            self.state.horizon_x = -4;
        } else {
            // Rendering not yet complete
            self.rendered = false;
            self.state.horizon_x += 1;
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

    #[cfg(feature = "vtx")]
    fn draw_vtx_channel(&mut self) -> bool {
        true
    }

    #[cfg(feature = "battery")]
    fn draw_current_draw(&mut self, draw_context: &OsdDrawContext) -> bool {
        let amperage = draw_context.battery_message.current.amperage_x100;
        _ = write!(self.fixed_buf, "{:3}{}", amperage, OsdSymbols::AMP);
        true
    }

    #[cfg(feature = "battery")]
    fn draw_mah_drawn(&mut self, draw_context: &OsdDrawContext) -> bool {
        let mah_drawn = draw_context.battery_message.current.mah_drawn;
        if mah_drawn >= <i16 as Into<i32>>::into(self.osd_cap_alarm) {
            self.attr = DisplayPortSeverity::Normal;
        }
        _ = write!(self.fixed_buf, "{:4}{}", mah_drawn, OsdSymbols::MAH);
        true
    }

    #[cfg(feature = "gps")]
    fn draw_gps_speed(&mut self) -> bool {
        true
    }

    #[cfg(feature = "gps")]
    fn draw_gps_sats(&mut self) -> bool {
        true
    }

    fn draw_altitude(&mut self) -> bool {
        self.fixed_buf[0] = OsdSymbols::ALTITUDE;
        self.fixed_buf[1] = OsdSymbols::HYPHEN;
        self.fixed_buf[2] = 0;
        true
    }

    pub fn format_pid(&mut self, label: &str, pid: PidConfig) {
        _ = write!(self.fixed_buf, "{} {:3} {:3} {:3} {:3} {:3}", label, pid.kp, pid.ki, pid.kd, pid.ks, pid.kk);
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

    fn draw_power(&mut self) -> bool {
        true
    }

    fn draw_pid_rate_profile(&mut self) -> bool {
        true
    }

    fn draw_warnings(&mut self) -> bool {
        true
    }

    fn draw_average_cell_voltage(&mut self) -> bool {
        true
    }

    #[cfg(feature = "gps")]
    fn draw_gps_lon(&mut self) -> bool {
        true
    }

    #[cfg(feature = "gps")]
    fn draw_gps_lat(&mut self) -> bool {
        true
    }

    fn draw_debug(&mut self) -> bool {
        let debug = GLOBAL_DEBUG.values();
        _ = write!(self.fixed_buf, "DBG {:5} {:5} {:5} {:5}", debug[0], debug[1], debug[2], debug[3]);
        true
    }

    fn draw_debug2(&mut self) -> bool {
        let debug = GLOBAL_DEBUG.values();
        _ = write!(self.fixed_buf, "DBG {:5} {:5} {:5} {:5}", debug[4], debug[5], debug[6], debug[7]);
        true
    }

    fn draw_pitch_angle(&mut self, angle_degrees: i32) -> bool {
        let sign_char = if angle_degrees < 0 { '-' } else { ' ' };
        let angle_abs = angle_degrees.unsigned_abs(); // Converts to unsigned, avoiding negation overflow
        _ = write!(self.fixed_buf, "{}{}{:02}", OsdSymbols::ROLL, sign_char, angle_abs);
        true
    }

    fn draw_roll_angle(&mut self, angle_degrees: i32) -> bool {
        // floor is supported natively on ARM Cortex-M, round is not
        let sign_char = if angle_degrees < 0 { '-' } else { ' ' };
        let angle_abs = angle_degrees.unsigned_abs(); // Converts to unsigned, avoiding negation overflow
        _ = write!(self.fixed_buf, "{}{}{:02}", OsdSymbols::ROLL, sign_char, angle_abs);
        true
    }

    fn draw_disarmed(&mut self, draw_context: &OsdDrawContext) -> bool {
        if !draw_context.arming_flags.is_set(ArmingFlags::ARMED) {
            self.write_string("DISARMED");
        }
        /*_ = self.write_custom(|w| {
            w.append_str_right_aligned("DISARMED", 8);
        });*/
        true
    }

    #[cfg(feature = "gps")]
    fn draw_home_direction(&mut self) -> bool {
        true
    }

    #[cfg(feature = "gps")]
    fn draw_home_distance(&mut self) -> bool {
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

    fn draw_numerical_vario(&mut self) -> bool {
        true
    }

    fn draw_compass_bar(&mut self) -> bool {
        true
    }

    #[cfg(feature = "dshot_telemetry")]
    fn draw_esc_temperature(&mut self) -> bool {
        true
    }

    #[cfg(feature = "dshot_telemetry")]
    fn draw_esc_rpm(&mut self) -> bool {
        true
    }

    fn draw_numerical_heading(&mut self, angle_degrees: i32) -> bool {
        _ = write!(self.fixed_buf, "{}{:03}", Self::direction_symbol_from_heading(angle_degrees), angle_degrees);
        true
    }

    fn draw_remaining_time_estimate(&mut self) -> bool {
        true
    }

    fn draw_rtc_date_time(&mut self) -> bool {
        true
    }

    fn draw_adjustment_range(&mut self) -> bool {
        true
    }

    fn draw_core_temperature(&mut self) -> bool {
        true
    }

    fn draw_anti_gravity(&mut self, draw_context: &OsdDrawContext) -> bool {
        if draw_context.rx_message.rc_modes.test(RcMode::ANTIGRAVITY) {
            self.write_string("AG");
        }
        true
    }

    fn draw_g_force(&mut self) -> bool {
        true
    }

    fn draw_motor_diagnostics(&mut self) -> bool {
        true
    }

    #[cfg(feature = "blackbox")]
    fn draw_log_status(&mut self) -> bool {
        true
    }

    fn draw_flip_arrow(&mut self) -> bool {
        true
    }

    fn draw_link_quality(&mut self) -> bool {
        true
    }

    #[cfg(feature = "gps")]
    fn draw_flight_distance(&mut self) -> bool {
        true
    }

    fn draw_stick_overlay(&mut self) -> bool {
        true
    }

    #[cfg(feature = "dshot_telemetry")]
    fn draw_esc_rpm_frequency(&mut self) -> bool {
        true
    }

    fn draw_rate_profile_name(&mut self) -> bool {
        true
    }

    fn draw_pid_profile_name(&mut self) -> bool {
        true
    }

    fn draw_profile_name(&mut self) -> bool {
        true
    }

    fn draw_rssi_dmb_value(&mut self) -> bool {
        true
    }

    fn draw_rc_channels(&mut self, rc_controls_pwm: [u16; 4], rc_channels: [i8; 4]) -> bool {
        let rc_channel_index = usize::from(self.state.rc_channel.min(3));

        let channel_pwm = rc_controls_pwm[rc_channel_index];
        if rc_channels[rc_channel_index] >= 0 {
            let channel = radio_controllers::RxChannel::map_rpy_pwm_to_plus_minus_1000(channel_pwm);
            _ = write!(self.fixed_buf, "{channel:5}");
            self.offset_y = self.state.rc_channel;
        }

        self.state.rc_channel += 1;
        if self.state.rc_channel == Osd::RC_CHANNELS_COUNT_U8 {
            self.state.rc_channel = 0;
        } else {
            // we have more channels to draw
            self.rendered = false;
        }
        true
    }

    #[cfg(feature = "gps")]
    fn draw_efficiency(&mut self) -> bool {
        true
    }

    fn draw_total_flights(&mut self) -> bool {
        true
    }

    fn draw_up_down_reference(&mut self) -> bool {
        true
    }

    fn draw_tx_uplink_power(&mut self) -> bool {
        true
    }

    fn draw_watt_hours_drawn(&mut self) -> bool {
        true
    }

    fn draw_aux_value(&mut self) -> bool {
        true
    }

    fn draw_ready_mode(&mut self) -> bool {
        true
    }

    fn draw_rsnr_value(&mut self) -> bool {
        true
    }

    #[cfg(feature = "gps")]
    fn draw_lap_time_current(&mut self) -> bool {
        true
    }

    #[cfg(feature = "gps")]
    fn draw_lap_time_previous(&mut self) -> bool {
        true
    }

    #[cfg(feature = "gps")]
    fn draw_lap_time_best3(&mut self) -> bool {
        true
    }

    fn draw_custom_message(&mut self) -> bool {
        true
    }

    #[cfg(feature = "rangefinder")]
    fn draw_lidar_distance(&mut self) -> bool {
        true
    }

    fn draw_custom_serial_text(&mut self) -> bool {
        true
    }

    fn draw_battery_profile_name(&mut self) -> bool {
        true
    }
}

// element background drawing functions
impl OsdElement {
    fn draw_background_horizon_sidebars<D: Display>(&mut self, display_port: &mut D) -> bool {
        const AH_SIDEBAR_WIDTH_POS: u8 = 7;
        const AH_SIDEBAR_HEIGHT_POS: i8 = 3;

        self.state.sidebar_render_level = false;
        self.state.sidebar_y = -AH_SIDEBAR_HEIGHT_POS;
        // Draw AH sides
        let hud_width = AH_SIDEBAR_WIDTH_POS;
        let hud_height = AH_SIDEBAR_HEIGHT_POS;

        if self.state.sidebar_render_level {
            // AH level indicators
            _ = display_port.write_byte(
                self.pos_x - hud_width + 1,
                self.pos_y,
                OsdSymbols::AH_LEFT,
                DisplayPortSeverity::Normal,
            );
            _ = display_port.write_byte(
                self.pos_x + hud_width - 1,
                self.pos_y,
                OsdSymbols::AH_RIGHT,
                DisplayPortSeverity::Normal,
            );
            self.state.sidebar_render_level = false;
        } else {
            _ = display_port.write_byte(
                self.pos_x - hud_width,
                (self.pos_y.cast_signed() + self.state.sidebar_y).cast_unsigned(),
                OsdSymbols::AH_DECORATION,
                DisplayPortSeverity::Normal,
            );
            _ = display_port.write_byte(
                self.pos_x + hud_width,
                (self.pos_y.cast_signed() + self.state.sidebar_y).cast_unsigned(),
                OsdSymbols::AH_DECORATION,
                DisplayPortSeverity::Normal,
            );

            if self.state.sidebar_y == hud_height {
                // Rendering is complete, so prepare to start again
                self.state.sidebar_y = -hud_height;
                // On next pass render the level markers
                self.state.sidebar_render_level = true;
            } else {
                self.state.sidebar_y += 1;
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
            self.write_slice(pilot_config.craft_name.as_slice());
        }
        true
    }

    fn draw_background_stick_overlay(&mut self) -> bool {
        const OSD_STICK_OVERLAY_WIDTH: usize = 7;
        const OSD_STICK_OVERLAY_HEIGHT: u8 = 5;

        if self.state.stick_overlay_render_phase == OsdStickOverlayRenderPhase::Vertical {
            self.fixed_buf[0] = OsdSymbols::STICK_OVERLAY_VERTICAL;
            self.offset_y = self.state.stick_overlay_y;
            self.state.stick_overlay_y += 1;
            if self.state.stick_overlay_y == (OSD_STICK_OVERLAY_HEIGHT - 1) / 2 {
                // Skip over horizontal
                self.state.stick_overlay_y += 1;
            }
            if self.state.stick_overlay_y == OSD_STICK_OVERLAY_HEIGHT {
                self.state.stick_overlay_y = 0;
                self.state.stick_overlay_render_phase = OsdStickOverlayRenderPhase::Horizontal;
            }
            self.rendered = false;
        } else {
            self.fixed_buf.bytes[..OSD_STICK_OVERLAY_WIDTH].fill(OsdSymbols::STICK_OVERLAY_HORIZONTAL);
            self.fixed_buf[(OSD_STICK_OVERLAY_WIDTH - 1) / 2] = OsdSymbols::STICK_OVERLAY_CENTER;
            self.fixed_buf[OSD_STICK_OVERLAY_WIDTH] = 0; // string terminator

            self.offset_y = (OSD_STICK_OVERLAY_HEIGHT - 1) / 2;
            self.state.stick_overlay_render_phase = OsdStickOverlayRenderPhase::Vertical;
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
            self.write_slice(pilot_config.pilot_name.as_slice());
        }
        true
    }

    fn draw_background_camera_frame<D: Display>(&mut self, display_port: &mut D, osd_config: &OsdConfig) -> bool {
        const OSD_CAMERA_FRAME_MIN_WIDTH: u8 = 2;
        const OSD_CAMERA_FRAME_MAX_WIDTH: u8 = 30; // Characters per row supported by MAX7456
        const OSD_CAMERA_FRAME_MIN_HEIGHT: u8 = 2;
        const OSD_CAMERA_FRAME_MAX_HEIGHT: u8 = 16; // Rows supported by MAX7456 (PAL)

        let xpos = self.pos_x;
        let ypos = self.pos_y;
        let width = osd_config.camera_frame_width.clamp(OSD_CAMERA_FRAME_MIN_WIDTH, OSD_CAMERA_FRAME_MAX_WIDTH);
        let height = osd_config.camera_frame_height.clamp(OSD_CAMERA_FRAME_MIN_HEIGHT, OSD_CAMERA_FRAME_MAX_HEIGHT);

        if self.state.camera_frame_render_phase != OsdStickCameraFrameRenderPhase::Bottom {
            // Rendering not yet complete
            self.rendered = false;
        }

        if self.state.camera_frame_render_phase == OsdStickCameraFrameRenderPhase::Middle {
            self.state.camera_frame_i = 1;

            _ = display_port.write_byte(
                xpos,
                ypos + self.state.camera_frame_i,
                OsdSymbols::STICK_OVERLAY_VERTICAL,
                DisplayPortSeverity::Normal,
            );
            _ = display_port.write_byte(
                xpos + width - 1,
                ypos + self.state.camera_frame_i,
                OsdSymbols::STICK_OVERLAY_VERTICAL,
                DisplayPortSeverity::Normal,
            );

            self.draw_element = false; // element already drawn

            self.state.camera_frame_i += 1;
            if self.state.camera_frame_i == height {
                self.state.camera_frame_i = 1;
                self.state.camera_frame_render_phase = OsdStickCameraFrameRenderPhase::Bottom;
            }
        } else {
            self.fixed_buf[0] = OsdSymbols::STICK_OVERLAY_CENTER;
            self.fixed_buf[1..(width as usize - 1)].fill(OsdSymbols::STICK_OVERLAY_HORIZONTAL);
            self.fixed_buf[width as usize - 1] = OsdSymbols::STICK_OVERLAY_CENTER;
            self.fixed_buf[width as usize] = 0; // string terminator

            if self.state.camera_frame_render_phase == OsdStickCameraFrameRenderPhase::Top {
                self.state.camera_frame_render_phase = OsdStickCameraFrameRenderPhase::Middle;
            } else {
                self.offset_y = height - 1;
                self.state.camera_frame_render_phase = OsdStickCameraFrameRenderPhase::Top;
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
