use radio_controllers::{Rates, RatesConfig, RcModes, RcModesArray};
use serde::{Deserialize, Serialize};
use stream_buf::{StreamBufReader, StreamBufWriter};
use vqm::Vector3di32;

use crate::config::{ConfigItem, ConfigPublisher, GLOBAL_CONFIG, GyroPidItem, GyroPidPublisher};

// return positive for ACK, negative on error, zero for no reply
pub enum MspResult {
    Ack = 1,
    Error = -1,
    #[allow(unused)]
    NoReply = 0,
    CmdUnknown = -2, // don't know how to process command, try next handler
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Msp {
    pub version: u8,
}

impl Msp {
    pub const fn new() -> Self {
        Self { version: 0 }
    }
}

impl Default for Msp {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused)]
impl Msp {
    const PID_CONTROLLER_BETAFLIGHT: u8 = 1;
    pub async fn process_write_command(cmd_msp: u16, dst: &mut StreamBufWriter<'_>) -> MspResult {
        match cmd_msp {
            Msp::API_VERSION => {
                #[allow(clippy::cast_possible_truncation)]
                {
                    dst.write_u8(Msp::PROTOCOL_VERSION as u8);
                    dst.write_u8(Msp::API_VERSION_MAJOR as u8);
                    dst.write_u8(Msp::API_VERSION_MINOR as u8);
                }
                MspResult::Ack
            }
            Msp::MODE_RANGES => Self::mode_ranges(dst).await,
            Msp::MODE_RANGES_EXTRA => Self::mode_ranges_extra(dst).await,
            Msp::MIXER_CONFIG => Self::mixer_config(dst).await,
            Msp::RX_CONFIG => Self::rx_config(dst).await,
            Msp::RSSI_CONFIG => Self::rssi_config(dst).await,
            Msp::PID_CONTROLLER => {
                dst.write_u8(Self::PID_CONTROLLER_BETAFLIGHT);
                MspResult::Ack
            }
            Msp::ALTITUDE => {
                dst.write_u32(0);
                dst.write_u16(0);
                MspResult::Ack
            }
            Msp::SONAR_ALTITUDE => {
                dst.write_u32(0);
                MspResult::Ack
            }
            Msp::FAILSAFE_CONFIG => Self::failsafe_config(dst).await,
            Msp::FILTER_CONFIG => Self::filter_config(dst).await,
            Msp::RAW_IMU => Self::raw_imu(dst).await,
            Msp::RC => Self::rc(dst).await,
            Msp::RC_TUNING => Self::rc_tuning(dst).await,
            Msp::PID => Self::pid(dst).await,
            #[cfg(feature = "gps")]
            Msp::GPS_CONFIG => Self::gps_config(dst).await,
            Msp::MOTOR_CONFIG
            | Msp::MOTOR_TELEMETRY
            | Msp::COMPASS_CONFIG
            | Msp::ACC_TRIM
            | Msp::MSP2_MOTOR_OUTPUT_REORDERING => MspResult::CmdUnknown,
            _ => {
                // we do not know how to handle the (valid) message, indicate an error MSP `$M!`.
                MspResult::Error
            }
        }
    }

    pub async fn process_read_command(
        cmd_msp: u16,
        src: &mut StreamBufReader<'_>,
        config_publisher: &ConfigPublisher<'_>,
        gyro_pid_publisher: &GyroPidPublisher<'_>,
    ) -> MspResult {
        match cmd_msp {
            Msp::SET_MODE_RANGE => Self::set_mode_range(src, config_publisher).await,
            Msp::SET_MIXER_CONFIG => Self::set_mixer_config(src, config_publisher).await,
            Msp::SET_RX_CONFIG => Self::set_rx_config(src, config_publisher).await,
            Msp::SET_RSSI_CONFIG => Self::set_rssi_config(src, config_publisher).await,
            Msp::SET_FAILSAFE_CONFIG => Self::set_failsafe_config(src, config_publisher).await,
            Msp::SET_FILTER_CONFIG => Self::set_filter_config(src, config_publisher).await,
            Msp::SET_RC_TUNING => Self::set_rc_tuning(src, config_publisher).await,
            Msp::SET_PID => Self::set_pid(src, gyro_pid_publisher).await,
            #[cfg(feature = "gps")]
            Msp::SET_GPS_CONFIG => Self::set_gps_config(src, config_publisher).await,
            _ => {
                // we do not know how to handle the (valid) message, indicate an error MSP `$M!`.
                MspResult::CmdUnknown
            }
        }
    }
}

impl Msp {
    async fn mode_ranges(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let rc_modes = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.rc_modes
        };
        for mac in rc_modes.macs {
            let Some(rc_mode) = RcModesArray::find_rc_mode_by_id(mac.mode_id) else { return MspResult::CmdUnknown };
            dst.write_u8(rc_mode.permanent_id);
            dst.write_u8(mac.aux_channel_index);
            dst.write_u8(mac.range.start);
            dst.write_u8(mac.range.end);
        }
        MspResult::Ack
    }
    async fn mode_ranges_extra(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let rc_modes = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.rc_modes
        };
        for mac in rc_modes.macs {
            let Some(rc_mode) = RcModesArray::find_rc_mode_by_id(mac.mode_id) else { return MspResult::CmdUnknown };
            let Some(linked_mode) = RcModesArray::find_rc_mode_by_id(mac.mode_id) else { return MspResult::CmdUnknown };
            dst.write_u8(rc_mode.permanent_id);
            dst.write_u8(mac.mode_logic);
            dst.write_u8(linked_mode.permanent_id);
        }
        MspResult::Ack
    }
    async fn set_mode_range(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        if src.bytes_remaining() < 1 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut rc_modes = global_config.rc_modes;
        let old_rc_modes = rc_modes;

        let mac_index = usize::from(src.read_u8());
        if mac_index >= RcModes::MAX_MODE_ACTIVATION_CONDITION_COUNT {
            return MspResult::Error;
        }
        let rc_mode_id = src.read_u8();
        let Some(rc_mode) = RcModesArray::find_rc_mode_by_id(rc_mode_id) else { return MspResult::CmdUnknown };

        let mut mac = rc_modes.mac(mac_index);
        mac.mode_id = rc_mode.id;
        mac.aux_channel_index = src.read_u8();
        mac.range.start = src.read_u8();
        mac.range.end = src.read_u8();

        if src.bytes_remaining() >= 2 {
            mac.mode_logic = src.read_u8();
            let linked_to_index = src.read_u8();
            let link = RcModesArray::find_rc_mode_by_permanent_id(linked_to_index);
            if let Some(rc_mode) = link {
                mac.linked_to = rc_mode.id;
            }
        }

        rc_modes.set_mac(mac_index, mac);
        rc_modes.analyze_macs();

        if rc_modes != old_rc_modes {
            global_config.rc_modes = rc_modes;
            publisher.publish(ConfigItem::RcModes(rc_modes)).await;
        }
        MspResult::Ack
    }

    async fn mixer_config(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let config = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.mixer
        };
        dst.write_u8(config.mixer_type);
        dst.write_u8(u8::from(config.yaw_motors_reversed));
        MspResult::Ack
    }
    async fn set_mixer_config(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        if src.bytes_remaining() < 1 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut config = global_config.mixer;
        let old_config = config;

        config.mixer_type = src.read_u8();
        if src.bytes_remaining() > 0 {
            config.yaw_motors_reversed = src.read_u8() != 0;
        }

        if config != old_config {
            global_config.mixer = config;
            publisher.publish(ConfigItem::Mixer(config)).await;
        }
        MspResult::Ack
    }

    async fn rssi_config(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let rx = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.rx
        };
        dst.write_u8(rx.rssi_channel);
        MspResult::Ack
    }
    async fn set_rssi_config(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        // 1. Check if enough data is even present before locking anything
        if src.bytes_remaining() < 1 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut rx = global_config.rx;
        let old_rx = rx;

        rx.rssi_channel = src.read_u8();
        if old_rx != rx {
            global_config.rx = rx;
            publisher.publish(ConfigItem::Rx(rx)).await;
        }
        MspResult::Ack
    }

    async fn rx_config(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let rx = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.rx
        };
        dst.write_u8(rx.serial_rx_provider);
        dst.write_u16(rx.max_check);
        dst.write_u16(rx.mid_rc);
        dst.write_u16(rx.min_check);
        dst.write_u8(rx.spektrum_sat_bind);
        dst.write_u16(rx.rx_min_us);
        dst.write_u16(rx.rx_max_us);
        dst.write_u8(0); // not required in API 1.44, was rx.rcInterpolation
        dst.write_u8(0); // not required in API 1.44, was rx.rcInterpolationInterval
        dst.write_u16(u16::from(rx.air_mode_activate_threshold) * 10 + 1000);
        dst.write_u8(0);
        dst.write_u32(0);
        dst.write_u8(0);
        dst.write_u8(rx.fpv_cam_angle_degrees);
        dst.write_u8(0); // not required in API 1.44, was rx.rcSmoothingChannels
        dst.write_u8(0);
        dst.write_u8(0);
        dst.write_u8(0);
        dst.write_u8(0);
        dst.write_u8(0);
        dst.write_u8(0);
        MspResult::Ack
    }
    async fn set_rx_config(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        // 1. Check if enough data is even present before locking anything
        if src.bytes_remaining() < 8 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut rx = global_config.rx;
        let old_rx = rx;

        rx.serial_rx_provider = src.read_u8();
        rx.max_check = src.read_u16();
        rx.mid_rc = src.read_u16();
        rx.min_check = src.read_u16();
        rx.spektrum_sat_bind = src.read_u8();
        if src.bytes_remaining() >= 4 {
            rx.rx_min_us = src.read_u16();
            rx.rx_max_us = src.read_u16();
        }
        if src.bytes_remaining() >= 4 {
            _ = src.read_u8(); // not required in API 1.44, was rx.rcInterpolation
            _ = src.read_u8(); // not required in API 1.44, was rx.rcInterpolationInterval
            #[allow(clippy::cast_possible_truncation)]
            {
                rx.air_mode_activate_threshold = ((src.read_u16() - 1000) / 10) as u8;
            }
        }
        if src.bytes_remaining() >= 6 {
            // skip RX_SPI values
            _ = src.read_u8();
            _ = src.read_u32();
            _ = src.read_u8();
        }
        if src.bytes_remaining() >= 1 {
            rx.fpv_cam_angle_degrees = src.read_u8();
        }

        if old_rx != rx {
            global_config.rx = rx;
            publisher.publish(ConfigItem::Rx(rx)).await;
        }
        MspResult::Ack
    }

    async fn failsafe_config(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let config = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.failsafe
        };
        dst.write_u8(config.delay_deciseconds);
        dst.write_u8(config.landing_time_seconds);
        dst.write_u16(config.throttle_pwm);
        dst.write_u8(config.switch_mode);
        dst.write_u16(config.throttle_low_delay_deciseconds);
        dst.write_u8(config.procedure);
        MspResult::Ack
    }
    async fn set_failsafe_config(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        // Check if enough data is even present before locking anything
        if src.bytes_remaining() < 8 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut config = global_config.failsafe;
        let old_config = config;

        config.delay_deciseconds = src.read_u8();
        config.landing_time_seconds = src.read_u8();
        config.throttle_pwm = src.read_u16();
        config.switch_mode = src.read_u8();
        config.throttle_low_delay_deciseconds = src.read_u16();
        config.procedure = src.read_u8();

        if config != old_config {
            global_config.failsafe = config;
            publisher.publish(ConfigItem::Failsafe(config)).await;
        }
        MspResult::Ack
    }

    async fn filter_config(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let (imu_filters, fc_filters) = {
            let global_config = GLOBAL_CONFIG.lock().await;
            (global_config.imu_filter_bank, global_config.flight_control_filters)
        };

        #[allow(clippy::cast_possible_truncation)]
        dst.write_u8(imu_filters.gyro_lpf1_hz as u8);
        dst.write_u16(fc_filters.dterm_lpf1_hz);
        dst.write_u16(fc_filters.yaw_lpf_hz);
        #[allow(clippy::cast_possible_truncation)]
        dst.write_u8(imu_filters.gyro_lpf1_hz as u8);
        dst.write_u16(fc_filters.dterm_lpf1_hz);
        dst.write_u16(fc_filters.yaw_lpf_hz);
        dst.write_u16(imu_filters.gyro_notch1_hz);
        dst.write_u16(imu_filters.gyro_notch1_cutoff);
        dst.write_u16(fc_filters.dterm_notch_hz);
        dst.write_u16(fc_filters.dterm_notch_cutoff);
        dst.write_u16(imu_filters.gyro_notch2_hz);
        dst.write_u16(imu_filters.gyro_notch2_cutoff);
        dst.write_u8(fc_filters.dterm_lpf1_type);
        dst.write_u8(0); // gyro_hardware_lpf set in driver
        dst.write_u8(0); // was gyro_32khz_hardware_lpf
        dst.write_u16(imu_filters.gyro_lpf1_hz);
        dst.write_u16(imu_filters.gyro_lpf2_hz);
        dst.write_u8(0); // TODO: imu_filters.gyro_lpf1_type);
        dst.write_u8(0); // TODO: imu_filters.gyro_lpf2_type);
        dst.write_u16(fc_filters.dterm_lpf2_hz);
        // Added in MSP API 1.41
        dst.write_u8(fc_filters.dterm_lpf2_type);
        dst.write_u16(0); // imu_filters.gyro_dynamic_lpf1_min_hz);
        dst.write_u16(0); //imu_filters.gyro_dynamic_lpf1_max_hz);
        dst.write_u16(fc_filters.dterm_dynamic_lpf1_min_hz);
        dst.write_u16(fc_filters.dterm_dynamic_lpf1_max_hz);
        // Added in MSP API 1.42
        dst.write_u8(0); // DEPRECATED 1.43: dyn_notch_range
        dst.write_u8(0); // DEPRECATED 1.44: dyn_notch_width_percent
        dst.write_u16(0); // dynNotchConfig.dyn_notch_q
        dst.write_u16(0); // dynNotchConfig.dyn_notch_min_hz
        #[cfg(feature = "rpm_filters")]
        dst.write_u8(imu_filters.rpm_filters.rpm_filter_harmonics);
        #[cfg(feature = "rpm_filters")]
        dst.write_u8(imu_filters.rpm_filters.rpm_filter_min_hz);
        // Added in MSP API 1.43
        dst.write_u16(0); // dynNotchConfig.dyn_notch_max_hz

        MspResult::Ack
    }
    async fn set_filter_config(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        // 1. Check if enough data is even present before locking anything
        if src.bytes_remaining() < 8 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut fc_filters = global_config.flight_control_filters;
        let old_fc_filters = fc_filters;
        let mut imu_filters = global_config.imu_filter_bank;
        let old_imu_filters = imu_filters;

        imu_filters.gyro_lpf1_hz = u16::from(src.read_u8());
        fc_filters.dterm_lpf1_hz = src.read_u16();
        fc_filters.yaw_lpf_hz = src.read_u16();
        if src.bytes_remaining() >= 8 {
            imu_filters.gyro_notch1_hz = src.read_u16();
            imu_filters.gyro_notch1_cutoff = src.read_u16();
            fc_filters.dterm_notch_hz = src.read_u16();
            fc_filters.dterm_notch_cutoff = src.read_u16();
        }
        if src.bytes_remaining() >= 4 {
            imu_filters.gyro_notch2_hz = src.read_u16();
            imu_filters.gyro_notch2_cutoff = src.read_u16();
        }
        if src.bytes_remaining() >= 1 {
            fc_filters.dterm_lpf1_type = src.read_u8();
        }
        if src.bytes_remaining() >= 10 {
            _ = src.read_u8(); // ignored gyro_hardware_lpf set in driver
            _ = src.read_u8(); // was gyro_32khz_hardware_lpf
            imu_filters.gyro_lpf1_hz = src.read_u16();
            imu_filters.gyro_lpf2_hz = src.read_u16();
            //imu_filters.gyro_lpf1_type = src.read_u8();
            //imu_filters.gyro_lpf2_type = src.read_u8();
            _ = src.read_u8();
            _ = src.read_u8();
            fc_filters.dterm_lpf2_hz = src.read_u16();
        }

        if src.bytes_remaining() >= 9 {
            // Added in MSP API 1.41
            fc_filters.dterm_lpf2_type = src.read_u8();
            /*imu_filters.gyro_dynamic_lpf1_min_hz = */
            _ = src.read_u16();
            /*imu_filters.gyro_dynamic_lpf1_max_hz = */
            _ = src.read_u16();
            fc_filters.dterm_dynamic_lpf1_min_hz = src.read_u16();
            fc_filters.dterm_dynamic_lpf1_max_hz = src.read_u16();
        }

        if src.bytes_remaining() >= 8 {
            // Added in MSP API 1.42
            _ = src.read_u8(); // DEPRECATED 1.43: dyn_notch_range
            _ = src.read_u8(); // DEPRECATED 1.44: dyn_notch_width_percent
            //dynamic_notch_q =
            _ = src.read_u16();
            //dynamic_notch_min_hz =
            _ = src.read_u16();
            #[cfg(feature = "rpm_filters")]
            {
                imu_filters.rpm_filters.rpm_filter_harmonics = src.read_u8();
                imu_filters.rpm_filters.rpm_filter_min_hz = src.read_u8();
            }
            #[cfg(not(feature = "rpm_filters"))]
            {
                _ = src.read_u16();
            }
        }
        if src.bytes_remaining() >= 2 {
            // Added in MSP API 1.43
            //dynamic_notch_max_hz =
            _ = src.read_u16();
        }
        if src.bytes_remaining() >= 2 {
            // Added in MSP API 1.44
            // dterm_lpf1_dyn_expo =
            _ = src.read_u8();
            // dynamic_notch_count =
            _ = src.read_u8();
        }

        if fc_filters != old_fc_filters {
            global_config.flight_control_filters = fc_filters;
            publisher.publish(ConfigItem::FlightControlFilters(fc_filters)).await;
        }
        if imu_filters != old_imu_filters {
            global_config.imu_filter_bank = imu_filters;
            publisher.publish(ConfigItem::ImuFilters(imu_filters)).await;
        }

        MspResult::Ack
    }

    async fn raw_imu(dst: &mut StreamBufWriter<'_>) -> MspResult {
        {
            let _ = GLOBAL_CONFIG.lock().await;
        }
        #[allow(clippy::cast_possible_truncation)]
        {
            let acc = Vector3di32::default();
            dst.write_u16(acc.x.cast_unsigned() as u16);
            dst.write_u16(acc.y.cast_unsigned() as u16);
            dst.write_u16(acc.z.cast_unsigned() as u16);
            let gyro = Vector3di32::default();
            dst.write_u16(gyro.x.cast_unsigned() as u16);
            dst.write_u16(gyro.y.cast_unsigned() as u16);
            dst.write_u16(gyro.z.cast_unsigned() as u16);
            let mag = Vector3di32::default();
            dst.write_u16(mag.x.cast_unsigned() as u16);
            dst.write_u16(mag.y.cast_unsigned() as u16);
            dst.write_u16(mag.z.cast_unsigned() as u16);
        }
        MspResult::Ack
    }

    async fn rc(_dst: &mut StreamBufWriter<'_>) -> MspResult {
        {
            let _ = GLOBAL_CONFIG.lock().await;
        }
        MspResult::Ack
    }

    async fn rc_tuning(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let rates = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.rates
        };
        dst.write_u8(rates.rc_rates[Rates::ROLL]);
        dst.write_u8(rates.rc_expos[Rates::ROLL]);
        dst.write_u8(rates.rates[Rates::ROLL]);
        dst.write_u8(rates.rates[Rates::PITCH]);
        dst.write_u8(rates.rates[Rates::YAW]);
        dst.write_u8(0); // was tpa_rate
        dst.write_u8(rates.throttle_midpoint);
        dst.write_u8(rates.throttle_expo);
        dst.write_u16(0); // was tpa_breakpoint
        dst.write_u8(rates.rc_expos[Rates::YAW]);
        dst.write_u8(rates.rc_rates[Rates::YAW]);
        dst.write_u8(rates.rc_rates[Rates::PITCH]);
        dst.write_u8(rates.rc_expos[Rates::PITCH]);
        // added in 1.41
        dst.write_u8(rates.throttle_limit_type);
        dst.write_u8(rates.throttle_limit_percent);
        // added in 1.42
        dst.write_u16(rates.limits[Rates::ROLL]);
        dst.write_u16(rates.limits[Rates::PITCH]);
        dst.write_u16(rates.limits[Rates::YAW]);
        // added in 1.43
        dst.write_u8(RatesConfig::TYPE_ACTUAL); // hardcoded, since we only support RATES_TYPE_ACTUAL rates.ratesType);
        MspResult::Ack
    }
    async fn set_rc_tuning(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        if src.bytes_remaining() < 10 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut rates = global_config.rates;
        let old_rates = rates;

        let value = src.read_u8();
        if rates.rc_rates[Rates::PITCH] == rates.rc_rates[Rates::ROLL] {
            rates.rc_rates[Rates::PITCH] = value;
        }
        rates.rc_rates[Rates::ROLL] = value;

        let value = src.read_u8();
        if rates.rc_expos[Rates::PITCH] == rates.rc_expos[Rates::ROLL] {
            rates.rc_expos[Rates::PITCH] = value;
        }
        rates.rc_expos[Rates::ROLL] = value;

        rates.rc_rates[Rates::ROLL] = src.read_u8();
        rates.rc_rates[Rates::PITCH] = src.read_u8();
        rates.rc_rates[Rates::YAW] = src.read_u8();
        _ = src.read_u8(); // skip tpa_rate
        rates.throttle_midpoint = src.read_u8();
        rates.throttle_expo = src.read_u8();
        _ = src.read_u16(); // skip tpa_breakpoint

        if src.bytes_remaining() >= 1 {
            rates.rc_expos[Rates::YAW] = src.read_u8();
        }
        if src.bytes_remaining() >= 1 {
            rates.rc_rates[Rates::YAW] = src.read_u8();
        }
        if src.bytes_remaining() >= 1 {
            rates.rc_rates[Rates::PITCH] = src.read_u8();
        }
        if src.bytes_remaining() >= 1 {
            rates.rc_expos[Rates::PITCH] = src.read_u8();
        }
        // version 1.41
        if src.bytes_remaining() >= 2 {
            rates.throttle_limit_type = src.read_u8();
            rates.throttle_limit_percent = src.read_u8();
        }
        // version 1.42
        if src.bytes_remaining() >= 6 {
            rates.limits[Rates::ROLL] = src.read_u16();
            rates.limits[Rates::PITCH] = src.read_u16();
            rates.limits[Rates::YAW] = src.read_u16();
        }
        // version 1.43
        if src.bytes_remaining() >= 1 {
            _ = src.read_u8(); // hardcoded to RATES_TYPE_ACTUAL
        }
        if rates != old_rates {
            global_config.rates = rates;
            publisher.publish(ConfigItem::Rates(rates)).await;
        }
        MspResult::Ack
    }

    async fn pid(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let (roll_rate, pitch_rate, yaw_rate, roll_angle, pitch_angle) = {
            let global_config = GLOBAL_CONFIG.lock().await;
            (
                global_config.pid_roll_rate,
                global_config.pid_pitch_rate,
                global_config.pid_yaw_rate,
                global_config.pid_roll_angle,
                global_config.pid_pitch_angle,
            )
        };
        dst.write_u8(roll_rate.kp);
        dst.write_u8(roll_rate.ki);
        dst.write_u8(roll_rate.kd);
        dst.write_u8(pitch_rate.kp);
        dst.write_u8(pitch_rate.ki);
        dst.write_u8(pitch_rate.kd);
        dst.write_u8(yaw_rate.kp);
        dst.write_u8(yaw_rate.ki);
        dst.write_u8(yaw_rate.kd);
        dst.write_u8(roll_angle.kp);
        dst.write_u8(roll_angle.ki);
        dst.write_u8(roll_angle.kd);
        // NOTE: Betaflight uses same values for roll_angle and pitch_angle pids, and saves mag_pid here
        dst.write_u8(pitch_angle.kp);
        dst.write_u8(pitch_angle.ki);
        dst.write_u8(pitch_angle.kd);
        MspResult::Ack
    }
    async fn set_pid(src: &mut StreamBufReader<'_>, publisher: &GyroPidPublisher<'_>) -> MspResult {
        if src.bytes_remaining() < 1 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;

        let mut roll_rate = global_config.pid_roll_rate;
        let old_roll_rate = roll_rate;
        roll_rate.kp = src.read_u8();
        roll_rate.ki = src.read_u8();
        roll_rate.kd = src.read_u8();
        if roll_rate != old_roll_rate {
            global_config.pid_roll_rate = roll_rate;
            publisher.publish(GyroPidItem::PitchRate(roll_rate)).await;
        }
        let mut pitch_rate = global_config.pid_pitch_rate;
        let old_pitch_rate = pitch_rate;
        pitch_rate.kp = src.read_u8();
        pitch_rate.ki = src.read_u8();
        pitch_rate.kd = src.read_u8();
        if pitch_rate != old_pitch_rate {
            global_config.pid_pitch_rate = pitch_rate;
            publisher.publish(GyroPidItem::PitchRate(pitch_rate)).await;
        }
        let mut yaw_rate = global_config.pid_yaw_rate;
        let old_yaw_rate = yaw_rate;
        yaw_rate.kp = src.read_u8();
        yaw_rate.ki = src.read_u8();
        yaw_rate.kd = src.read_u8();
        if yaw_rate != old_yaw_rate {
            global_config.pid_yaw_rate = yaw_rate;
            publisher.publish(GyroPidItem::PitchRate(yaw_rate)).await;
        }
        let mut roll_angle = global_config.pid_roll_angle;
        let old_roll_angle = roll_angle;
        roll_angle.kp = src.read_u8();
        roll_angle.ki = src.read_u8();
        roll_angle.kd = src.read_u8();
        if roll_angle != old_roll_angle {
            global_config.pid_roll_angle = roll_angle;
            publisher.publish(GyroPidItem::PitchRate(roll_angle)).await;
        }
        let mut pitch_angle = global_config.pid_pitch_angle;
        let old_pitch_angle = pitch_angle;
        pitch_angle.kp = src.read_u8();
        pitch_angle.ki = src.read_u8();
        pitch_angle.kd = src.read_u8();
        if pitch_angle != old_pitch_angle {
            global_config.pid_pitch_angle = pitch_angle;
            publisher.publish(GyroPidItem::PitchRate(pitch_angle)).await;
        }

        MspResult::Ack
    }

    #[cfg(feature = "gps")]
    async fn gps_config(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let config = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.gps
        };
        dst.write_u8(config.provider);
        dst.write_u8(config.sbas_mode);
        dst.write_u8(config.auto_config);
        dst.write_u8(config.auto_baud);
        // Added in API version 1.43
        dst.write_u8(config.gps_set_home_point_once);
        dst.write_u8(config.gps_ublox_use_galileo);

        MspResult::Ack
    }
    #[cfg(feature = "gps")]
    async fn set_gps_config(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        if src.bytes_remaining() < 4 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut config = global_config.gps;
        let old_config = config;

        config.provider = src.read_u8();
        config.sbas_mode = src.read_u8();
        config.auto_config = src.read_u8();
        config.auto_baud = src.read_u8();
        if src.bytes_remaining() >= 2 {
            // Added in API version 1.43
            config.gps_set_home_point_once = src.read_u8();
            config.gps_ublox_use_galileo = src.read_u8();
        }

        if config != old_config {
            global_config.gps = config;
            publisher.publish(ConfigItem::Gps(config)).await;
        }
        MspResult::Ack
    }
}
