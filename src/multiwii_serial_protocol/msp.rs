use radio_controllers::{Rates, RatesConfig, RcModes, RcModesArray};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use stream_buf::{StreamBufReader, StreamBufWriter};
use vqm::{Quaternion, Vector3di16};

use crate::config::{ConfigItem, ConfigPublisher, FastConfigItem, FastConfigPublisher, GLOBAL_CONFIG};
#[cfg(feature = "gps")]
use crate::gps::GpsSolutionDataAbridged;

// return positive for ACK, negative on error, zero for no reply
pub enum MspResult {
    Ack = 1,
    Error = -1,
    #[allow(unused)]
    NoReply = 0,
    /// Don't know how to process command, so try next handler.
    CmdUnknown = -2,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MspSensorData {
    #[cfg(feature = "barometer")]
    pub barometer_altitude_cm: u32,
    #[cfg(feature = "rangefinder")]
    pub rangefinder_altitude_cm: u32,
    pub attitude: Vector3di16,
    // an unexpected use of generic - I didn't expect it might be used this way when I wrote the quaternion code!.
    pub attitude_quaternion: Quaternion<u16>,
    pub acc: Vector3di16,
    pub gyro: Vector3di16,
    pub mag: Vector3di16,
    #[cfg(feature = "gps")]
    pub gps_sol: GpsSolutionDataAbridged,
}

impl MspSensorData {
    pub const fn new() -> Self {
        Self {
            #[cfg(feature = "barometer")]
            barometer_altitude_cm: 0,
            #[cfg(feature = "rangefinder")]
            rangefinder_altitude_cm: 0,
            attitude: Vector3di16 { x: 0, y: 0, z: 0 },
            attitude_quaternion: Quaternion::<u16> { w: 0, x: 0, y: 0, z: 0 },
            acc: Vector3di16 { x: 0, y: 0, z: 0 },
            gyro: Vector3di16 { x: 0, y: 0, z: 0 },
            mag: Vector3di16 { x: 0, y: 0, z: 0 },
            #[cfg(feature = "gps")]
            gps_sol: GpsSolutionDataAbridged::new(),
        }
    }
}
/// MSP configurator. Reads and writes data in Betaflight MSP-compatible format.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

    /// Write the configuration data to the MSP stream.
    pub async fn process_write_command(
        cmd_msp: u16,
        dst: &mut StreamBufWriter<'_>,
        sensor_data: &MspSensorData,
    ) -> MspResult {
        match cmd_msp {
            #[allow(clippy::cast_possible_truncation)]
            Msp::API_VERSION => {
                dst.write_u8(Msp::PROTOCOL_VERSION as u8);
                dst.write_u8(Msp::API_VERSION_MAJOR as u8);
                dst.write_u8(Msp::API_VERSION_MINOR as u8);
                MspResult::Ack
            }
            Msp::MODE_RANGES => Self::mode_ranges(dst).await,
            Msp::MODE_RANGES_EXTRA => Self::mode_ranges_extra(dst).await,
            Msp::FEATURE_CONFIG => Self::feature_config(dst).await,
            Msp::MIXER_CONFIG => Self::mixer_config(dst).await,
            Msp::RX_CONFIG => Self::rx_config(dst).await,
            Msp::RSSI_CONFIG => Self::rssi_config(dst).await,
            Msp::PID_CONTROLLER => {
                dst.write_u8(Self::PID_CONTROLLER_BETAFLIGHT);
                MspResult::Ack
            }
            #[cfg(feature = "rangefinder")]
            Msp::SONAR_ALTITUDE => {
                dst.write_u32(sensor_data.rangefinder_altitude_cm);
                MspResult::Ack
            }
            Msp::ARMING_CONFIG => Self::arming_config(dst).await,
            Msp::FAILSAFE_CONFIG => Self::failsafe_config(dst).await,
            Msp::ADVANCED_CONFIG => Self::advanced_config(dst).await,
            Msp::FILTER_CONFIG => Self::filter_config(dst).await,
            Msp::SENSOR_CONFIG => Self::sensor_config(dst).await,
            Msp::STATUS => Self::status(dst).await,
            Msp::RAW_IMU => Self::raw_imu(dst, sensor_data),
            Msp::RC => Self::rc(dst).await,
            #[cfg(feature = "barometer")]
            Msp::ALTITUDE => {
                dst.write_u32(sensor_data.barometer_altitude_cm);
                dst.write_u16(0);
                MspResult::Ack
            }
            Msp::ATTITUDE => {
                dst.write_u16(sensor_data.attitude.x.cast_unsigned());
                dst.write_u16(sensor_data.attitude.y.cast_unsigned());
                dst.write_u16(sensor_data.attitude.z.cast_unsigned());
                MspResult::Ack
            }
            Msp::ANALOG => {
                // TODO: Msp:ANALOG
                dst.write_u8(0); // legacy battery voltage
                dst.write_u16(0); // mAh drawn from battery
                dst.write_u16(0); // RSSI
                dst.write_u16(0); // current in 0.01 A steps, range is -320A to 320A
                dst.write_u16(0); // battery voltage
                MspResult::Ack
            }
            Msp::ATTITUDE_QUATERNION => {
                dst.write_u16(sensor_data.attitude_quaternion.w);
                dst.write_u16(sensor_data.attitude_quaternion.x);
                dst.write_u16(sensor_data.attitude_quaternion.y);
                dst.write_u16(sensor_data.attitude_quaternion.z);
                MspResult::Ack
            }
            Msp::RC_TUNING => Self::rc_tuning(dst).await,
            Msp::PID => Self::pid(dst).await,

            #[cfg(feature = "magnetometer")]
            Msp::COMPASS_CONFIG => Self::compass_config(dst).await,

            #[cfg(feature = "battery")]
            Msp::BATTERY_CONFIG => Self::battery_config(dst).await,

            #[cfg(feature = "gps")]
            Msp::GPS_RESCUE => Self::gps_rescue(dst).await,

            #[cfg(feature = "gps")]
            Msp::RAW_GPS => Self::raw_gps(dst, sensor_data),

            #[cfg(feature = "gps")]
            Msp::GPS_CONFIG => Self::gps_config(dst).await,

            Msp::MOTOR_CONFIG => Self::motor_config(dst).await,
            Msp::RC_DEADBAND => Self::rc_controls_config(dst).await,

            Msp::MOTOR_TELEMETRY | Msp::COMPASS_CONFIG | Msp::ACC_TRIM | Msp::MSP2_MOTOR_OUTPUT_REORDERING => {
                MspResult::CmdUnknown
            }

            _ => {
                // we do not know how to handle the (valid) message, indicate an error MSP `$M!`.
                MspResult::Error
            }
        }
    }

    /// Read in configuration from MSP stream.
    /// If the configuration has changed then publish it to either the `config_publisher` or the `fast_config_publisher`
    /// depending on the configuration settings.
    /// Not that the is a check to see if the configuration has changed before it is published,
    /// Than makes each `set_config` function less efficient, but it means the higher priority tasks
    /// don't need to waste there time processing unnecessary messages.
    pub async fn process_read_command(
        cmd_msp: u16,
        src: &mut StreamBufReader<'_>,
        config_publisher: &ConfigPublisher<'_>,
        fast_config_publisher: &FastConfigPublisher<'_>,
    ) -> MspResult {
        match cmd_msp {
            Msp::SET_MODE_RANGE => Self::set_mode_range(src, config_publisher).await,
            Msp::SET_FEATURE_CONFIG => Self::set_feature_config(src, config_publisher).await,
            Msp::SET_MIXER_CONFIG => Self::set_mixer_config(src, config_publisher).await,
            Msp::SET_RX_CONFIG => Self::set_rx_config(src, config_publisher).await,
            Msp::SET_RSSI_CONFIG => Self::set_rssi_config(src, config_publisher).await,
            Msp::SET_ARMING_CONFIG => Self::set_arming_config(src, config_publisher).await,
            Msp::SET_FAILSAFE_CONFIG => Self::set_failsafe_config(src, config_publisher).await,
            Msp::SET_ADVANCED_CONFIG => Self::set_advanced_config(src, config_publisher).await,
            Msp::SET_FILTER_CONFIG => Self::set_filter_config(src, config_publisher).await,
            Msp::RC_DEADBAND => Self::set_rc_controls_config(src, config_publisher).await,
            Msp::SET_RC_TUNING => Self::set_rc_tuning(src, config_publisher).await,
            Msp::SET_PID => Self::set_pid(src, fast_config_publisher).await,
            Msp::SELECT_SETTING => Self::select_setting(src),
            Msp::SET_HEADING => Self::set_heading(src),
            Msp::SET_MOTOR_CONFIG => Self::set_motor_config(src, config_publisher).await,

            #[cfg(feature = "magnetometer")]
            Msp::SET_COMPASS_CONFIG => Self::set_compass_config(src, config_publisher).await,

            #[cfg(feature = "battery")]
            Msp::SET_BATTERY_CONFIG => Self::set_battery_config(src, config_publisher).await,

            #[cfg(feature = "gps")]
            Msp::SET_GPS_CONFIG => Self::set_gps_config(src, config_publisher).await,

            #[cfg(feature = "gps")]
            Msp::SET_GPS_RESCUE => Self::set_gps_rescue(src, config_publisher).await,

            _ => {
                // we do not know how to handle the (valid) message, indicate an error MSP `$M!`.
                MspResult::CmdUnknown
            }
        }
    }
}

impl Msp {
    async fn feature_config(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let feature = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.features
        };
        dst.write_u32(feature.flags());
        MspResult::Ack
    }
    async fn set_feature_config(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        // Check if enough data is even present before locking anything
        if src.bytes_remaining() < 4 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut features = global_config.features;

        features.set_flags(src.read_u32());
        if features != global_config.features {
            global_config.features = features;
            publisher.publish(ConfigItem::Features(features)).await;
        }
        MspResult::Ack
    }

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

        if rc_modes != global_config.rc_modes {
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
        dst.write_u8(config.yaw_motors_reversed);
        MspResult::Ack
    }
    async fn set_mixer_config(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        if src.bytes_remaining() < 1 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut config = global_config.mixer;

        config.mixer_type = src.read_u8();
        if src.bytes_remaining() > 0 {
            config.yaw_motors_reversed = src.read_u8();
        }

        if config != global_config.mixer {
            global_config.mixer = config;
            publisher.publish(ConfigItem::Mixer(config)).await;
        }
        MspResult::Ack
    }

    fn select_setting(_src: &mut StreamBufReader<'_>) -> MspResult {
        /*const RATE_PROFILE_MASK:u8 = 1 << 7;
        const BATTERY_PROFILE_MASK:u8 = 1 << 6;
        let mut value = src.read_u8();
        if value & BATTERY_PROFILE_MASK != 0 {
            value = (value & !BATTERY_PROFILE_MASK).clamp(0, BatteryProfiles::MAX);
            change_battery_profile(value);
        } else if value & RATE_PROFILE_MASK == 0 {
            if not armed ) {
                value = value.clamp(0, 4);
                change_pid_profile(value);
            }
        } else {
            value = (value & !RATE_PROFILE_MASK).clamp(0, 3);
            change_rate_profile(value);
        }*/
        MspResult::Ack
    }

    async fn motor_config(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let config = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.motor
        };
        dst.write_u16(0); // min_throttle deprecated in 4.6
        dst.write_u16(config.max_throttle);
        dst.write_u16(config.min_command);
        dst.write_u8(config.motor_pole_count);
        dst.write_u8(0); // use_dshot_telemetry
        MspResult::Ack
    }
    async fn set_motor_config(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        if src.bytes_remaining() < 6 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut config = global_config.motor;

        _ = src.read_u16(); // min_throttle deprecated in 4.6
        config.max_throttle = src.read_u16();
        config.min_command = src.read_u16();

        // version 1.42
        if src.bytes_remaining() > 2 {
            config.motor_pole_count = src.read_u8();
            _ = src.read_u8(); // TODO: use_dshot_telemetry
        }

        if config != global_config.motor {
            global_config.motor = config;
            publisher.publish(ConfigItem::Motor(config)).await;
        }
        MspResult::Ack
    }

    #[cfg(feature = "magnetometer")]
    async fn compass_config(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let config = {
            let global_config = GLOBAL_CONFIG.lock().await;
            global_config.imu
        };
        dst.write_u16(config.mag_declination_degrees_x10.cast_unsigned());
        MspResult::Ack
    }
    #[cfg(feature = "magnetometer")]
    async fn set_compass_config(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut config = global_config.imu;

        config.mag_declination_degrees_x10 = src.read_u16().cast_signed();

        if config != global_config.imu {
            global_config.imu = config;
            publisher.publish(ConfigItem::Imu(config)).await;
        }
        MspResult::Ack
    }

    async fn rc_controls_config(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let (rc_controls, position_hold) = {
            let global_config = GLOBAL_CONFIG.lock().await;
            (global_config.rc_controls, global_config.position_hold)
        };
        dst.write_u8(rc_controls.deadband);
        dst.write_u8(rc_controls.yaw_deadband);
        dst.write_u8(position_hold.deadband);
        dst.write_u16(50); // TODO: deadband3d throttle from flight3D config
        MspResult::Ack
    }
    async fn set_rc_controls_config(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        if src.bytes_remaining() < 2 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut rc_controls = global_config.rc_controls;
        let mut position_hold = global_config.position_hold;

        rc_controls.deadband = src.read_u8();
        rc_controls.yaw_deadband = src.read_u8();

        if src.bytes_remaining() > 1 {
            position_hold.deadband = src.read_u8();
        }
        if src.bytes_remaining() > 1 {
            _ = src.read_u8();
        }

        if rc_controls != global_config.rc_controls {
            global_config.rc_controls = rc_controls;
            publisher.publish(ConfigItem::RcControls(rc_controls)).await;
        }
        if position_hold != global_config.position_hold {
            global_config.position_hold = position_hold;
            publisher.publish(ConfigItem::PositionHold(position_hold)).await;
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

        rx.rssi_channel = src.read_u8();
        if rx != global_config.rx {
            global_config.rx = rx;
            publisher.publish(ConfigItem::Rx(rx)).await;
        }
        MspResult::Ack
    }

    async fn arming_config(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let (arming_config, imu_config) = {
            let global_config = GLOBAL_CONFIG.lock().await;
            (global_config.arming, global_config.imu)
        };
        dst.write_u8(arming_config.auto_disarm_delay);
        dst.write_u8(0); // was disarm_kill_switch
        dst.write_u8(imu_config.small_angle);
        dst.write_u8(arming_config.gyro_cal_on_first_arm);
        MspResult::Ack
    }
    async fn set_arming_config(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        // Check if enough data is even present before locking anything
        if src.bytes_remaining() < 1 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut arming_config = global_config.arming;
        let mut imu_config = global_config.imu;

        arming_config.auto_disarm_delay = src.read_u8();
        _ = src.read_u8(); // disarm_kill_switch has been removed
        if src.bytes_remaining() < 1 {
            imu_config.small_angle = src.read_u8();
        }
        if src.bytes_remaining() < 1 {
            arming_config.gyro_cal_on_first_arm = src.read_u8();
        }
        if global_config.arming != arming_config {
            global_config.arming = arming_config;
            publisher.publish(ConfigItem::Arming(arming_config)).await;
        }
        if imu_config != global_config.imu {
            global_config.imu = imu_config;
            publisher.publish(ConfigItem::Imu(imu_config)).await;
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
        // Check if enough data is even present before locking anything.
        if src.bytes_remaining() < 8 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut rx = global_config.rx;

        rx.serial_rx_provider = src.read_u8();
        rx.max_check = src.read_u16();
        rx.mid_rc = src.read_u16();
        rx.min_check = src.read_u16();
        rx.spektrum_sat_bind = src.read_u8();
        if src.bytes_remaining() >= 4 {
            rx.rx_min_us = src.read_u16();
            rx.rx_max_us = src.read_u16();
        }
        #[allow(clippy::cast_possible_truncation)]
        if src.bytes_remaining() >= 4 {
            _ = src.read_u8(); // not required in API 1.44, was rx.rcInterpolation
            _ = src.read_u8(); // not required in API 1.44, was rx.rcInterpolationInterval
            rx.air_mode_activate_threshold = ((src.read_u16() - 1000) / 10) as u8;
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

        if rx != global_config.rx {
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

        config.delay_deciseconds = src.read_u8();
        config.landing_time_seconds = src.read_u8();
        config.throttle_pwm = src.read_u16();
        config.switch_mode = src.read_u8();
        config.throttle_low_delay_deciseconds = src.read_u16();
        config.procedure = src.read_u8();

        if config != global_config.failsafe {
            global_config.failsafe = config;
            publisher.publish(ConfigItem::Failsafe(config)).await;
        }
        MspResult::Ack
    }

    async fn advanced_config(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let (gyro, motor, motor_device, system) = {
            let global_config = GLOBAL_CONFIG.lock().await;
            (global_config.gyro, global_config.motor, global_config.motor_device, global_config.system)
        };
        dst.write_u8(1); // was gyro_sync_denom - removed in API 1.43
        // dst.write_u8(pid.pid_process_denom); TODO: pid process denom in MSP
        dst.write_u8(1);
        dst.write_u8(motor_device.use_continuous_update);
        dst.write_u8(motor_device.motor_protocol);
        dst.write_u16(motor_device.motor_pwm_rate);
        dst.write_u16(motor.motor_idle);
        dst.write_u8(0); // Deprecated: gyro_use_32kHz
        dst.write_u8(motor_device.motor_inversion);
        dst.write_u8(0); // Deprecated: gyro_to_use
        dst.write_u8(gyro.gyro_high_fsr);
        dst.write_u8(gyro.gyro_movement_calibration_threshold);
        dst.write_u16(gyro.gyro_calibration_duration);
        dst.write_u16(gyro.gyro_offset_yaw.cast_unsigned());
        dst.write_u8(gyro.check_overflow);
        //Added in MSP API 1.42
        dst.write_u8(system.debug_mode);
        dst.write_u8(8); // TODO: DEBUG_COUNT;

        MspResult::Ack
    }
    async fn set_advanced_config(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        // Check if enough data is even present before locking anything
        if src.bytes_remaining() < 8 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut gyro = global_config.gyro;
        let mut motor = global_config.motor;
        let mut motor_device = global_config.motor_device;
        let mut system = global_config.system;

        _ = src.read_u8(); // was gyro_sync_denom - removed in API 1.43
        _ = src.read_u8(); // pid_process_denom
        motor_device.use_continuous_update = src.read_u8();
        motor_device.motor_protocol = src.read_u8();
        motor_device.motor_pwm_rate = src.read_u16();
        motor.motor_idle = src.read_u16();
        _ = src.read_u8(); // Deprecated: gyro_use_32kHz
        motor_device.motor_inversion = src.read_u8();
        _ = src.read_u8(); // Deprecated gyro_to_use
        gyro.gyro_high_fsr = src.read_u8();
        gyro.gyro_movement_calibration_threshold = src.read_u8();
        gyro.gyro_calibration_duration = src.read_u16();
        gyro.gyro_offset_yaw = src.read_u16().cast_signed();
        gyro.check_overflow = src.read_u8();
        if src.bytes_remaining() >= 1 {
            system.debug_mode = src.read_u8();
        }

        if gyro != global_config.gyro {
            global_config.gyro = gyro;
            publisher.publish(ConfigItem::Gyro(gyro)).await;
        }
        if motor != global_config.motor {
            global_config.motor = motor;
            publisher.publish(ConfigItem::Motor(motor)).await;
        }
        if motor_device != global_config.motor_device {
            global_config.motor_device = motor_device;
            publisher.publish(ConfigItem::MotorDevice(motor_device)).await;
        }
        if system != global_config.system {
            global_config.system = system;
            publisher.publish(ConfigItem::System(system)).await;
        }
        MspResult::Ack
    }

    async fn sensor_config(dst: &mut StreamBufWriter<'_>) -> MspResult {
        #[allow(unused)]
        let global_config = GLOBAL_CONFIG.lock().await;
        dst.write_u8(0); // acc hardware
        #[cfg(feature = "barometer")]
        dst.write_u8(global_config.barometer.hardware);
        #[cfg(not(feature = "barometer"))]
        dst.write_u8(0);

        // Added in MSP API 1.46
        #[cfg(feature = "rangefinder")]
        dst.write_u8(global_config.rangefinder.hardware);
        #[cfg(not(feature = "rangefinder"))]
        dst.write_u8(0);
        #[cfg(feature = "optical_flow")]
        dst.write_u8(global_config.optical_flow.hardware);
        #[cfg(not(feature = "optical_flow"))]
        dst.write_u8(0);

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
        dst.write_u8(0); // Deprecated 1.43: dyn_notch_range
        dst.write_u8(0); // Deprecated 1.44: dyn_notch_width_percent
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
        let mut imu_filters = global_config.imu_filter_bank;

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
            _ = src.read_u8(); // Deprecated 1.43: dyn_notch_range
            _ = src.read_u8(); // Deprecated 1.44: dyn_notch_width_percent
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

        if fc_filters != global_config.flight_control_filters {
            global_config.flight_control_filters = fc_filters;
            publisher.publish(ConfigItem::FlightControlFilters(fc_filters)).await;
        }
        if imu_filters != global_config.imu_filter_bank {
            global_config.imu_filter_bank = imu_filters;
            publisher.publish(ConfigItem::ImuFilters(imu_filters)).await;
        }

        MspResult::Ack
    }

    // TODO: MSP status placeholder
    async fn status(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let (sensors, pid_profile_index) = {
            let global_config = GLOBAL_CONFIG.lock().await;
            (global_config.sensors, global_config.system.pid_profile_index)
        };
        dst.write_u16(0); // pid task delta time
        dst.write_u16(0); // I2C error counter
        dst.write_u16(sensors.flags()); // sensors
        dst.write_u32(0); // flightmode flags
        dst.write_u8(pid_profile_index);

        MspResult::Ack
    }

    fn raw_imu(dst: &mut StreamBufWriter<'_>, sensor_data: &MspSensorData) -> MspResult {
        dst.write_u16(sensor_data.acc.x.cast_unsigned());
        dst.write_u16(sensor_data.acc.y.cast_unsigned());
        dst.write_u16(sensor_data.acc.z.cast_unsigned());
        dst.write_u16(sensor_data.gyro.x.cast_unsigned());
        dst.write_u16(sensor_data.gyro.y.cast_unsigned());
        dst.write_u16(sensor_data.gyro.z.cast_unsigned());
        dst.write_u16(sensor_data.mag.x.cast_unsigned());
        dst.write_u16(sensor_data.mag.y.cast_unsigned());
        dst.write_u16(sensor_data.mag.z.cast_unsigned());
        MspResult::Ack
    }

    async fn rc(_dst: &mut StreamBufWriter<'_>) -> MspResult {
        // Dummy await statement to satisfy the compiler and yield control
        // Will be removed once function fully implemented.
        embassy_time::Timer::after_ticks(0).await;
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
        if rates != global_config.rates {
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
    async fn set_pid(src: &mut StreamBufReader<'_>, publisher: &FastConfigPublisher<'_>) -> MspResult {
        if src.bytes_remaining() < 1 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;

        let mut roll_rate = global_config.pid_roll_rate;
        roll_rate.kp = src.read_u8();
        roll_rate.ki = src.read_u8();
        roll_rate.kd = src.read_u8();
        if roll_rate != global_config.pid_roll_rate {
            global_config.pid_roll_rate = roll_rate;
            publisher.publish(FastConfigItem::PitchRate(roll_rate)).await;
        }
        let mut pitch_rate = global_config.pid_pitch_rate;
        pitch_rate.kp = src.read_u8();
        pitch_rate.ki = src.read_u8();
        pitch_rate.kd = src.read_u8();
        if pitch_rate != global_config.pid_pitch_rate {
            global_config.pid_pitch_rate = pitch_rate;
            publisher.publish(FastConfigItem::PitchRate(pitch_rate)).await;
        }
        let mut yaw_rate = global_config.pid_yaw_rate;
        yaw_rate.kp = src.read_u8();
        yaw_rate.ki = src.read_u8();
        yaw_rate.kd = src.read_u8();
        if yaw_rate != global_config.pid_yaw_rate {
            global_config.pid_yaw_rate = yaw_rate;
            publisher.publish(FastConfigItem::PitchRate(yaw_rate)).await;
        }
        let mut roll_angle = global_config.pid_roll_angle;
        roll_angle.kp = src.read_u8();
        roll_angle.ki = src.read_u8();
        roll_angle.kd = src.read_u8();
        if roll_angle != global_config.pid_roll_angle {
            global_config.pid_roll_angle = roll_angle;
            publisher.publish(FastConfigItem::PitchRate(roll_angle)).await;
        }
        let mut pitch_angle = global_config.pid_pitch_angle;
        pitch_angle.kp = src.read_u8();
        pitch_angle.ki = src.read_u8();
        pitch_angle.kd = src.read_u8();
        if pitch_angle != global_config.pid_pitch_angle {
            global_config.pid_pitch_angle = pitch_angle;
            publisher.publish(FastConfigItem::PitchRate(pitch_angle)).await;
        }

        MspResult::Ack
    }

    fn set_heading(src: &mut StreamBufReader<'_>) -> MspResult {
        if src.bytes_remaining() < 2 {
            return MspResult::Error;
        }
        _ = src.read_u16();
        // TODO: update the sensor fusion filter with the new heading
        MspResult::Ack
    }

    #[cfg(feature = "battery")]
    async fn battery_config(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let (config, profiles) = {
            let global_config = GLOBAL_CONFIG.lock().await;
            (global_config.battery, global_config.battery_profiles)
        };

        let profile = profiles.profiles[0];
        #[allow(clippy::cast_possible_truncation)]
        {
            dst.write_u8(((profile.min_cell_voltage_v_x100 + 5) / 10) as u8);
            dst.write_u8(((profile.max_cell_voltage_v_x100 + 5) / 10) as u8);
            dst.write_u8(((profile.warning_cell_voltage_v_x100 + 5) / 10) as u8);
        }
        dst.write_u16(profile.battery_capacity_mah);
        dst.write_u8(config.voltage_meter_source);
        dst.write_u8(config.current_meter_source);
        dst.write_u16(profile.min_cell_voltage_v_x100);
        dst.write_u16(profile.max_cell_voltage_v_x100);
        dst.write_u16(profile.warning_cell_voltage_v_x100);

        MspResult::Ack
    }
    #[cfg(feature = "battery")]
    async fn set_battery_config(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        use crate::sensors::{CurrentMeterReading, VoltageMeterReading};

        if src.bytes_remaining() < 7 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut config = global_config.battery;
        let profile_index = global_config.system.active_battery_profile() as usize;
        let mut profile = global_config.battery_profiles[profile_index];

        let mut min_voltage = u16::from(src.read_u8()) * 10 - 5;
        let mut max_voltage = u16::from(src.read_u8()) * 10 - 5;
        let mut warning_voltage = u16::from(src.read_u8()) * 10 - 5;
        config.voltage_meter_source = src.read_u8().clamp(0, VoltageMeterReading::SOURCE_MAX);
        config.current_meter_source = src.read_u8().clamp(0, CurrentMeterReading::SOURCE_MAX);
        if src.bytes_remaining() >= 6 {
            min_voltage = src.read_u16();
            max_voltage = src.read_u16();
            warning_voltage = src.read_u16();
        }
        if config != global_config.battery {
            global_config.battery = config;
            publisher.publish(ConfigItem::Battery(config)).await;
        }
        if !(min_voltage..=max_voltage).contains(&warning_voltage) {
            return MspResult::Error;
        }
        profile.min_cell_voltage_v_x100 = min_voltage;
        profile.max_cell_voltage_v_x100 = max_voltage;
        profile.warning_cell_voltage_v_x100 = warning_voltage;
        if profile != global_config.battery_profiles.profiles[profile_index] {
            global_config.battery_profiles[profile_index] = profile;
            // TODO: publisher.publish(ConfigItem::BatteryProfiles(profile)).await;
        }
        MspResult::Ack
    }

    #[cfg(feature = "gps")]
    fn raw_gps(dst: &mut StreamBufWriter<'_>, sensor_data: &MspSensorData) -> MspResult {
        dst.write_u8(0); //STATE(GPS_FIX));
        dst.write_u8(sensor_data.gps_sol.satellite_count);
        dst.write_u32(sensor_data.gps_sol.llh.latitude_degrees_x1e7.cast_unsigned());
        dst.write_u32(sensor_data.gps_sol.llh.longitude_degrees_x1e7.cast_unsigned());
        // Altitude changed from 1m to 0.01m per lsb since MSP API 1.39 by RTH.
        // To maintain backwards compatibility compensate to 1m per lsb in MSP.
        #[allow(clippy::cast_possible_truncation)]
        dst.write_u16((sensor_data.gps_sol.llh.altitude_cm / 100).cast_unsigned() as u16);
        dst.write_u16(sensor_data.gps_sol.ground_speed_cmps);
        dst.write_u16(sensor_data.gps_sol.ground_course_degrees_x10);
        // Added in API version 1.44
        dst.write_u16(sensor_data.gps_sol.dop_positional);
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

        config.provider = src.read_u8();
        config.sbas_mode = src.read_u8();
        config.auto_config = src.read_u8();
        config.auto_baud = src.read_u8();
        if src.bytes_remaining() >= 2 {
            // Added in API version 1.43
            config.gps_set_home_point_once = src.read_u8();
            config.gps_ublox_use_galileo = src.read_u8();
        }

        if config != global_config.gps {
            global_config.gps = config;
            publisher.publish(ConfigItem::Gps(config)).await;
        }
        MspResult::Ack
    }

    #[cfg(feature = "gps")]
    async fn gps_rescue(dst: &mut StreamBufWriter<'_>) -> MspResult {
        let (gps_rescue_config, autopilot_config) = {
            let global_config = GLOBAL_CONFIG.lock().await;
            (global_config.gps_rescue, global_config.autopilot)
        };
        dst.write_u16(gps_rescue_config.max_rescue_angle_degrees);
        dst.write_u16(gps_rescue_config.return_altitude_m);
        dst.write_u16(gps_rescue_config.descent_distance_m);
        dst.write_u16(gps_rescue_config.ground_speed_cmps);
        dst.write_u16(autopilot_config.throttle_min);
        dst.write_u16(autopilot_config.throttle_max);
        dst.write_u16(autopilot_config.hover_throttle);
        dst.write_u8(gps_rescue_config.sanity_checks);
        dst.write_u8(gps_rescue_config.min_sats);
        // Added in API version 1.43
        dst.write_u16(gps_rescue_config.ascend_rate);
        dst.write_u16(gps_rescue_config.descend_rate);
        dst.write_u8(gps_rescue_config.allow_arming_without_fix);
        dst.write_u8(gps_rescue_config.altitude_mode);
        // Added in API version 1.44
        dst.write_u16(gps_rescue_config.min_start_dist_m);
        // Added in API version 1.46
        dst.write_u16(gps_rescue_config.initial_climb_m);

        MspResult::Ack
    }
    #[cfg(feature = "gps")]
    async fn set_gps_rescue(src: &mut StreamBufReader<'_>, publisher: &ConfigPublisher<'_>) -> MspResult {
        if src.bytes_remaining() < 18 {
            return MspResult::Error;
        }
        let mut global_config = GLOBAL_CONFIG.lock().await;
        let mut gps_rescue_config = global_config.gps_rescue;
        let mut autopilot_config = global_config.autopilot;

        gps_rescue_config.max_rescue_angle_degrees = src.read_u16();
        gps_rescue_config.return_altitude_m = src.read_u16();
        gps_rescue_config.descent_distance_m = src.read_u16();
        gps_rescue_config.ground_speed_cmps = src.read_u16();
        autopilot_config.throttle_min = src.read_u16();
        autopilot_config.throttle_max = src.read_u16();
        autopilot_config.hover_throttle = src.read_u16();
        gps_rescue_config.sanity_checks = src.read_u8();
        gps_rescue_config.min_sats = src.read_u8();

        if src.bytes_remaining() >= 6 {
            // Added in API version 1.43
            gps_rescue_config.ascend_rate = src.read_u16();
            gps_rescue_config.descend_rate = src.read_u16();
            gps_rescue_config.allow_arming_without_fix = src.read_u8();
            gps_rescue_config.altitude_mode = src.read_u8();
        }
        if src.bytes_remaining() >= 2 {
            // Added in API version 1.44
            gps_rescue_config.min_start_dist_m = src.read_u16();
        }
        if src.bytes_remaining() >= 2 {
            // Added in API version 1.46
            gps_rescue_config.initial_climb_m = src.read_u16();
        }

        if gps_rescue_config != global_config.gps_rescue {
            global_config.gps_rescue = gps_rescue_config;
            publisher.publish(ConfigItem::GpsRescue(gps_rescue_config)).await;
        }
        if autopilot_config != global_config.autopilot {
            global_config.autopilot = autopilot_config;
            publisher.publish(ConfigItem::Autopilot(autopilot_config)).await;
        }
        MspResult::Ack
    }
}
