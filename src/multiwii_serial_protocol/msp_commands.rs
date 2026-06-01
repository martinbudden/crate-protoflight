use crate::multiwii_serial_protocol::msp::Msp;

#[rustfmt::skip]
#[allow(unused)]
impl Msp {
    pub const PROTOCOL_VERSION: u16         = 0;
    pub const API_VERSION_MAJOR: u16        = 1; // increment when major changes are made
    pub const API_VERSION_MINOR: u16        = 47; // increment after a release; to set the version for all changes to go into the following release (if no changes to MSP are made between the releases; this can be reverted before the release)
    pub const API_VERSION_LENGTH: u16       = 2;

    pub const API_VERSION: u16              = 1;   // out message: Get API version
    pub const FC_VARIANT: u16               = 2;   // out message: Get flight controller variant
    pub const FC_VERSION: u16               = 3;   // out message: Get flight controller version
    pub const BOARD_INFO: u16               = 4;   // out message: Get board information
    pub const BUILD_INFO: u16               = 5;   // out message: Get build information

    pub const NAME: u16                     = 10;  // out message: Returns user set board name - betaflight
    pub const SET_NAME: u16                 = 11;  // in message:  Sets board name - betaflight

// Cleanflight original features (32-62)
    pub const BATTERY_CONFIG: u16           = 32;  // out message: Get battery configuration
    pub const SET_BATTERY_CONFIG: u16       = 33;  // in message:  Set battery configuration
    pub const MODE_RANGES: u16              = 34;  // out message: Returns all mode ranges
    pub const SET_MODE_RANGE: u16           = 35;  // in message:  Sets a single mode range
    pub const FEATURE_CONFIG: u16           = 36;  // out message: Get feature configuration
    pub const SET_FEATURE_CONFIG: u16       = 37;  // in message:  Set feature configuration
    pub const BOARD_ALIGNMENT_CONFIG: u16   = 38;  // out message: Get board alignment configuration
    pub const SET_BOARD_ALIGNMENT_CONFIG:u16= 39;  // in message:  Set board alignment configuration
    pub const CURRENT_METER_CONFIG: u16     = 40;  // out message: Get current meter configuration
    pub const SET_CURRENT_METER_CONFIG: u16 = 41;  // in message:  Set current meter configuration
    pub const MIXER_CONFIG: u16             = 42;  // out message: Get mixer configuration
    pub const SET_MIXER_CONFIG: u16         = 43;  // in message:  Set mixer configuration
    pub const RX_CONFIG: u16                = 44;  // out message: Get RX configuration
    pub const SET_RX_CONFIG: u16            = 45;  // in message:  Set RX configuration
    pub const LED_COLORS: u16               = 46;  // out message: Get LED colors
    pub const SET_LED_COLORS: u16           = 47;  // in message:  Set LED colors
    pub const LED_STRIP_CONFIG: u16         = 48;  // out message: Get LED strip configuration
    pub const SET_LED_STRIP_CONFIG: u16     = 49;  // in message:  Set LED strip configuration
    pub const RSSI_CONFIG: u16              = 50;  // out message: Get RSSI configuration
    pub const SET_RSSI_CONFIG: u16          = 51;  // in message:  Set RSSI configuration
    pub const ADJUSTMENT_RANGES: u16        = 52;  // out message: Get adjustment ranges
    pub const SET_ADJUSTMENT_RANGE: u16     = 53;  // in message:  Set adjustment range
    pub const CF_SERIAL_CONFIG: u16         = 54;  // out message: Get Cleanflight serial configuration
    pub const SET_CF_SERIAL_CONFIG: u16     = 55;  // in message:  Set Cleanflight serial configuration
    pub const VOLTAGE_METER_CONFIG: u16     = 56;  // out message: Get voltage meter configuration
    pub const SET_VOLTAGE_METER_CONFIG: u16 = 57;  // in message:  Set voltage meter configuration
    pub const SONAR_ALTITUDE: u16           = 58;  // out message: Get sonar altitude [cm]
    pub const PID_CONTROLLER: u16           = 59;  // out message: Get PID controller
    pub const SET_PID_CONTROLLER: u16       = 60;  // in message:  Set PID controller
    pub const ARMING_CONFIG: u16            = 61;  // out message: Get arming configuration
    pub const SET_ARMING_CONFIG: u16        = 62;  // in message:  Set arming configuration

// Baseflight MSP commands (64-89)
    pub const RX_MAP: u16                   = 64;  // out message: Get RX map (also returns number of channels total)
    pub const SET_RX_MAP: u16               = 65;  // in message:  Set RX map, channels count to set comes from MSP_RX_MAP
    // DEPRECATED - DO NOT USE "BF_CONFIG" and SET_BF_CONFIG.  In Cleanflight; isolated commands already exist and should be used instead.
    // DEPRECATED - BF_CONFIG: u16          = 66;  // out message baseflight-specific settings that aren't covered elsewhere
    // DEPRECATED - SET_BF_CONFIG: u16      = 67;  // in message baseflight-specific settings save
    pub const REBOOT: u16                   = 68;  // in message:  Reboot settings
    // Use BUILD_INFO instead
    // DEPRECATED - BF_BUILD_INFO: u16      = 69;  // out message build date as well as some space for future expansion
    pub const DATAFLASH_SUMMARY: u16        = 70;  // out message: Get description of dataflash chip
    pub const DATAFLASH_READ: u16           = 71;  // out message: Get content of dataflash chip
    pub const DATAFLASH_ERASE: u16          = 72;  // in message:  Erase dataflash chip
    // No-longer use
    // DEPRECATED - LOOP_TIME:u8            = 73;  // out message         Returns FC cycle time i.e loop time parameter // DEPRECATED
    // DEPRECATED - SET_LOOP_TIME:u8        = 74;  // in message          Sets FC cycle time i.e loop time parameter    // DEPRECATED
    pub const FAILSAFE_CONFIG: u16          = 75;  // out message: Get failsafe settings
    pub const SET_FAILSAFE_CONFIG: u16      = 76;  // in message:  Set failsafe settings
    pub const RXFAIL_CONFIG: u16            = 77;  // out message: Get RX failsafe settings
    pub const SET_RXFAIL_CONFIG: u16        = 78;  // in message:  Set RX failsafe settings
    pub const SDCARD_SUMMARY: u16           = 79;  // out message: Get SD card state
    pub const BLACKBOX_CONFIG: u16          = 80;  // out message: Get blackbox settings
    pub const SET_BLACKBOX_CONFIG: u16      = 81;  // in message:  Set blackbox settings
    pub const TRANSPONDER_CONFIG: u16       = 82;  // out message: Get transponder settings
    pub const SET_TRANSPONDER_CONFIG: u16   = 83;  // in message:  Set transponder settings
    pub const OSD_CONFIG: u16               = 84;  // out message: Get OSD settings
    pub const SET_OSD_CONFIG: u16           = 85;  // in message:  Set OSD settings
    pub const OSD_CHAR_READ: u16            = 86;  // out message: Get OSD characters
    pub const OSD_CHAR_WRITE: u16           = 87;  // in message:  Set OSD characters
    pub const VTX_CONFIG: u16               = 88;  // out message: Get VTX settings
    pub const SET_VTX_CONFIG: u16           = 89;  // in message:  Set VTX settings

// Betaflight Additional Commands (90-99)
    pub const ADVANCED_CONFIG: u16          = 90;  // out message: Get advanced configuration
    pub const SET_ADVANCED_CONFIG: u16      = 91;  // in message:  Set advanced configuration
    pub const FILTER_CONFIG: u16            = 92;  // out message: Get filter configuration
    pub const SET_FILTER_CONFIG: u16        = 93;  // in message:  Set filter configuration
    pub const PID_ADVANCED: u16             = 94;  // out message: Get advanced PID settings
    pub const SET_PID_ADVANCED: u16         = 95;  // in message:  Set advanced PID settings
    pub const SENSOR_CONFIG: u16            = 96;  // out message: Get sensor configuration
    pub const SET_SENSOR_CONFIG: u16        = 97;  // in message:  Set sensor configuration
    pub const CAMERA_CONTROL: u16           = 98;  // in/out message: Camera control
    pub const SET_ARMING_DISABLED: u16      = 99;  // in message:  Enable/disable arming

// Multiwii original MSP commands (101-139)
    //DEPRECATED - IDENT: u16               = 100; // out message         mixerMode + multiwii version + protocol version + capability variable
    pub const STATUS: u16                   = 101; // out message: Cycle time & errors_count & sensor present & box activation & current setting number
    pub const RAW_IMU: u16                  = 102; // out message: 9 DOF
    pub const SERVO: u16                    = 103; // out message: Servos
    pub const MOTOR: u16                    = 104; // out message: Motors
    pub const RC: u16                       = 105; // out message: RC channels and more
    pub const RAW_GPS: u16                  = 106; // out message: Fix, satellite count, lat, lon, alt, speed, ground course
    pub const COMP_GPS: u16                 = 107; // out message: Distance home, direction home
    pub const ATTITUDE: u16                 = 108; // out message: 2 angles 1 heading
    pub const ALTITUDE: u16                 = 109; // out message: Altitude, variometer
    pub const ANALOG: u16                   = 110; // out message: Vbat, power meter sum, rssi if available on RX
    pub const RC_TUNING: u16                = 111; // out message: RC rate, rc expo, roll_pitch rate, yaw rate, dyn throttle PID
    pub const PID: u16                      = 112; // out message: P I D coeff (9 are used currently)
    // Legacy MultiWi command that was never used.
    //DEPRECATED - BOX:u16                  = 113; // out message         BOX setup (number is dependant of your setup)
    // Legacy command that was under constant change due to the naming vagueness; avoid at all costs - use more specific commands instead.
    //DEPRECATED - MISC:u16                 = 114; // out message         power meter trig
    // Legacy MultiWi command that was never used and always wrong
    //DEPRECATED - MOTOR_PINS:u16           = 115; // out message         which pins are in use for motors & servos; for GUI
    pub const BOX_NAMES: u16                = 116; // out message: The aux switch names
    pub const PID_NAMES: u16                = 117; // out message: The PID names
    pub const WP: u16                       = 118; // out message: Get a WP, WP# is in the payload, returns (WP#, lat, lon, alt, flags) WP#0-home, WP#16-pos_hold
    pub const BOX_IDS: u16                  = 119; // out message: Get the permanent IDs associated to BOXes
    pub const SERVO_CONFIGURATIONS: u16     = 120; // out message: All servo configurations
    pub const NAV_STATUS: u16               = 121; // out message: Returns navigation status
    pub const NAV_CONFIG: u16               = 122; // out message: Returns navigation parameters
    pub const MOTOR_3D_CONFIG: u16          = 124; // out message: Settings needed for reversible ESCs
    pub const RC_DEADBAND: u16              = 125; // out message: Deadbands for yaw alt pitch roll
    pub const SENSOR_ALIGNMENT: u16         = 126; // out message: Orientation of acc,gyro,mag
    pub const LED_STRIP_MODE_COLOR: u16     = 127; // out message: Get LED strip mode_color settings
    pub const VOLTAGE_METERS: u16           = 128; // out message: Voltage (per meter)
    pub const CURRENT_METERS: u16           = 129; // out message: Amperage (per meter)
    pub const BATTERY_STATE: u16            = 130; // out message: Connected/Disconnected, Voltage, Current Used
    pub const MOTOR_CONFIG: u16             = 131; // out message: Motor configuration (min/max throttle, etc)
    pub const GPS_CONFIG: u16               = 132; // out message: GPS configuration
    pub const COMPASS_CONFIG: u16           = 133; // out message: Compass configuration
    pub const ESC_SENSOR_DATA: u16          = 134; // out message: Extra ESC data from 32-Bit ESCs (Temperature, RPM)
    pub const GPS_RESCUE: u16               = 135; // out message: GPS Rescue angle, returnAltitude, descentDistance, groundSpeed, sanityChecks and minSats
    pub const GPS_RESCUE_PIDS: u16          = 136; // out message: GPS Rescue throttleP and velocity PIDS + yaw P
    pub const VTX_TABLE_BAND: u16           = 137; // out message: VTX table band/channel data
    pub const VTX_TABLE_POWERLEVEL: u16     = 138; // out message: VTX table powerLevel data
    pub const MOTOR_TELEMETRY: u16          = 139; // out message: Per-motor telemetry data (RPM, packet stats, ESC temp, etc.)

// Simplified tuning commands (140-145)
    pub const SIMPLIFIED_TUNING: u16        = 140; // out message: Get simplified tuning values and enabled state
    pub const SET_SIMPLIFIED_TUNING: u16    = 141; // in message:  Set simplified tuning positions and apply calculated tuning
    pub const CALCULATE_SIMPLIFIED_PID: u16 = 142; // out message: Calculate PID values based on sliders without saving
    pub const CALCULATE_SIMPLIFIED_GYRO: u16= 143; // out message: Calculate gyro filter values based on sliders without saving
    pub const CALCULATE_SIMPLIFIED_DTERM:u16= 144; // out message: Calculate D term filter values based on sliders without saving
    pub const VALIDATE_SIMPLIFIED_TUNING:u16= 145; // out message: Returns array of true/false showing which simplified tuning groups match values

// Additional non-MultiWii commands (150-166)
    pub const STATUS_EX: u16                = 150; // out message: Cycle time, errors_count, CPU load, sensor present etc
    pub const UID: u16                      = 160; // out message: Unique device ID
    pub const GPS_SV_INFO: u16              = 164; // out message: Get Signal Strength (only U-Blox)
    pub const GPS_STATISTICS: u16           = 166; // out message: Get GPS debugging data
    pub const ATTITUDE_QUATERNION: u16      = 167; // out message: Orientation quaternion components (w, x, y, z)

// OSD specific commands (180-189)
    pub const OSD_VIDEO_CONFIG: u16         = 180; // out message: Get OSD video settings
    pub const SET_OSD_VIDEO_CONFIG: u16     = 181; // in message:  Set OSD video settings
    pub const DISPLAYPORT: u16              = 182; // out message: External OSD displayport mode
    pub const COPY_PROFILE: u16             = 183; // in message:  Copy settings between profiles
    pub const BEEPER_CONFIG: u16            = 184; // out message: Get beeper configuration
    pub const SET_BEEPER_CONFIG: u16        = 185; // in message:  Set beeper configuration
    pub const SET_TX_INFO: u16              = 186; // in message:  Set runtime information from TX lua scripts
    pub const TX_INFO: u16                  = 187; // out message: Get runtime information for TX lua scripts
    pub const SET_OSD_CANVAS: u16           = 188; // in message:  Set OSD canvas size COLSxROWS
    pub const OSD_CANVAS: u16               = 189; // out message: Get OSD canvas size COLSxROWS

// Set commands (200-229)
    pub const SET_RAW_RC: u16               = 200; // in message:  8 rc chan
    pub const SET_RAW_GPS: u16              = 201; // in message:  Fix, satellite count, lat, lon, alt, speed
    pub const SET_PID: u16                  = 202; // in message:  P I D coeff (9 are used currently)
    pub const SET_RC_TUNING: u16            = 204; // in message:  RC rate, rc expo, roll_pitch rate, yaw rate, dyn throttle PID, yaw expo
    pub const ACC_CALIBRATION: u16          = 205; // in message:  No param - calibrate accelerometer
    pub const MAG_CALIBRATION: u16          = 206; // in message:  No param - calibrate magnetometer
    pub const RESET_CONF: u16               = 208; // in message:  No param - reset settings
    pub const SET_WP: u16                   = 209; // in message:  Sets a given WP (WP#,lat, lon, alt, flags)
    pub const SELECT_SETTING: u16           = 210; // in message:  Select setting number (0-2)
    pub const SET_HEADING: u16              = 211; // in message:  Define a new heading hold direction
    pub const SET_SERVO_CONFIGURATION: u16  = 212; // in message:  Servo settings
    pub const SET_MOTOR: u16                = 214; // in message:  PropBalance function
    pub const SET_NAV_CONFIG: u16           = 215; // in message:  Sets nav config parameters
    pub const SET_MOTOR_3D_CONFIG: u16      = 217; // in message:  Settings needed for reversible ESCs
    pub const SET_RC_DEADBAND: u16          = 218; // in message:  Deadbands for yaw alt pitch roll
    pub const SET_RESET_CURR_PID: u16       = 219; // in message:  Reset current PID profile to defaults
    pub const SET_SENSOR_ALIGNMENT: u16     = 220; // in message:  Set the orientation of acc,gyro,mag
    pub const SET_LED_STRIP_MODE_COLOR: u16 = 221; // in message:  Set LED strip mode_color settings
    pub const SET_MOTOR_CONFIG: u16         = 222; // in message:  Motor configuration (min/max throttle, etc)
    pub const SET_GPS_CONFIG: u16           = 223; // in message:  GPS configuration
    pub const SET_COMPASS_CONFIG: u16       = 224; // in message:  Compass configuration
    pub const SET_GPS_RESCUE: u16           = 225; // in message:  Set GPS Rescue parameters
    pub const SET_GPS_RESCUE_PIDS: u16      = 226; // in message:  Set GPS Rescue PID values
    pub const SET_VTX_TABLE_BAND: u16       = 227; // in message:  Set vtxTable band/channel data
    pub const SET_VTX_TABLE_POWERLEVEL: u16 = 228; // in message:  Set vtxTable powerLevel data

// Multiple MSP and special commands (230-255)
    pub const MULTIPLE_MSP: u16             = 230; // out message: Request multiple MSPs in one request
    pub const MODE_RANGES_EXTRA: u16        = 238; // out message: Extra mode range data
    pub const SET_ACC_TRIM: u16             = 239; // in message:  Set acc angle trim values
    pub const ACC_TRIM: u16                 = 240; // out message: Get acc angle trim values
    pub const SERVO_MIX_RULES: u16          = 241; // out message: Get servo mixer configuration
    pub const SET_SERVO_MIX_RULE: u16       = 242; // in message:  Set servo mixer configuration
    pub const SET_PASSTHROUGH : u16         = 245; // in message:  Set passthrough to peripherals
    pub const SET_RTC: u16                  = 246; // in message:  Set the RTC clock
    pub const RTC: u16                      = 247; // out message: Get the RTC clock
    pub const SET_BOARD_INFO: u16           = 248; // in message:  Set the board information
    pub const SET_SIGNATURE: u16            = 249; // in message:  Set the signature of the board and serial number
    pub const EEPROM_WRITE: u16             = 250; // in message:  Write settings to EEPROM
    pub const RESERVE_1: u16                = 251; // reserved for system usage
    pub const RESERVE_2: u16                = 252; // reserved for system usage
    pub const DEBUG_MSG: u16                = 253; // out message: debug string buffer
    pub const DEBUG: u16                    = 254; // out message: debug1,debug2,debug3,debug4
    pub const V2_FRAME: u16                 = 255; // MSPv2 payload indicator


    pub const MSP2_COMMON_SERIAL_CONFIG: u16 = 0x1009;
    pub const MSP2_COMMON_SET_SERIAL_CONFIG: u16 = 0x100A;

// Sensors
    pub const MSP2_SENSOR_GPS: u16 = 0x1F03;

    pub const MSP2_BETAFLIGHT_BIND: u16 = 0x3000;
    pub const MSP2_MOTOR_OUTPUT_REORDERING: u16 = 0x3001;
    pub const MSP2_SET_MOTOR_OUTPUT_REORDERING: u16 = 0x3002;
    pub const MSP2_SEND_DSHOT_COMMAND: u16 = 0x3003;
    pub const MSP2_GET_VTX_DEVICE_STATUS: u16 = 0x3004;
    pub const MSP2_GET_OSD_WARNINGS: u16 = 0x3005; // returns active OSD warning message text
    pub const MSP2_GET_TEXT: u16 = 0x3006;
    pub const MSP2_SET_TEXT: u16 = 0x3007;
    pub const MSP2_GET_LED_STRIP_CONFIG_VALUES: u16 = 0x3008;
    pub const MSP2_SET_LED_STRIP_CONFIG_VALUES: u16 = 0x3009;
    pub const MSP2_SENSOR_CONFIG_ACTIVE: u16 = 0x300A;

// MSP2_SET_TEXT and MSP2_GET_TEXT variable types
    pub const MSP2TEXT_PILOT_NAME: u16 = 1;
    pub const MSP2TEXT_CRAFT_NAME: u16 = 2;
    pub const MSP2TEXT_PID_PROFILE_NAME: u16 = 3;
    pub const MSP2TEXT_RATE_PROFILE_NAME: u16 = 4;
    pub const MSP2TEXT_BUILD_KEY: u16 = 5;
    pub const MSP2TEXT_RELEASE_NAME: u16 = 6;
}
