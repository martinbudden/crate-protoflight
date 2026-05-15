use crate::multiwii_serial_protocol::msp::Msp;

#[allow(unused)]
impl Msp {
    pub const PROTOCOL_VERSION: u16 = 0;

    pub const API_VERSION_MAJOR: u16 = 1; // increment when major changes are made
    pub const API_VERSION_MINOR: u16 = 47; // increment after a release; to set the version for all changes to go into the following release (if no changes to MSP are made between the releases; this can be reverted before the release)

    pub const API_VERSION_LENGTH: u16 = 2;

    pub const API_VERSION: u16 = 1; //out message
    pub const FC_VARIANT: u16 = 2; //out message
    pub const FC_VERSION: u16 = 3; //out message
    pub const BOARD_INFO: u16 = 4; //out message
    pub const BUILD_INFO: u16 = 5; //out message

    pub const NAME: u16 = 10; //out message          Returns user set board name - betaflight
    pub const SET_NAME: u16 = 11; //in message           Sets board name - betaflight

    //
    // MSP commands for Cleanflight original features
    //
    pub const BATTERY_CONFIG: u16 = 32;
    pub const SET_BATTERY_CONFIG: u16 = 33;

    pub const MODE_RANGES: u16 = 34; //out message         Returns all mode ranges
    pub const SET_MODE_RANGE: u16 = 35; //in message          Sets a single mode range

    pub const FEATURE_CONFIG: u16 = 36;
    pub const SET_FEATURE_CONFIG: u16 = 37;

    pub const BOARD_ALIGNMENT_CONFIG: u16 = 38;
    pub const SET_BOARD_ALIGNMENT_CONFIG: u16 = 39;

    pub const CURRENT_METER_CONFIG: u16 = 40;
    pub const SET_CURRENT_METER_CONFIG: u16 = 41;

    pub const MIXER_CONFIG: u16 = 42;
    pub const SET_MIXER_CONFIG: u16 = 43;

    pub const RX_CONFIG: u16 = 44;
    pub const SET_RX_CONFIG: u16 = 45;

    pub const LED_COLORS: u16 = 46;
    pub const SET_LED_COLORS: u16 = 47;

    pub const LED_STRIP_CONFIG: u16 = 48;
    pub const SET_LED_STRIP_CONFIG: u16 = 49;

    pub const RSSI_CONFIG: u16 = 50;
    pub const SET_RSSI_CONFIG: u16 = 51;

    pub const ADJUSTMENT_RANGES: u16 = 52;
    pub const SET_ADJUSTMENT_RANGE: u16 = 53;

    // private - only to be used by the configurator; the commands are likely to change
    pub const CF_SERIAL_CONFIG: u16 = 54;
    pub const SET_CF_SERIAL_CONFIG: u16 = 55;

    pub const VOLTAGE_METER_CONFIG: u16 = 56;
    pub const SET_VOLTAGE_METER_CONFIG: u16 = 57;

    pub const SONAR_ALTITUDE: u16 = 58; //out message get sonar altitude [cm]

    pub const PID_CONTROLLER: u16 = 59;
    pub const SET_PID_CONTROLLER: u16 = 60;

    pub const ARMING_CONFIG: u16 = 61;
    pub const SET_ARMING_CONFIG: u16 = 62;

    //
    // Baseflight MSP commands (if enabled they exist in Cleanflight)
    //
    pub const RX_MAP: u16 = 64; //out message get channel map (also returns number of channels total)
    pub const SET_RX_MAP: u16 = 65; //in message set rx map; number of channels to set comes from RX_MAP

    // DEPRECATED - DO NOT USE "BF_CONFIG" and SET_BF_CONFIG.  In Cleanflight; isolated commands already exist and should be used instead.
    // DEPRECATED - pub const BF_CONFIG                  :u8 =66 //out message baseflight-specific settings that aren't covered elsewhere
    // DEPRECATED - pub const SET_BF_CONFIG              :u8 =67 //in message baseflight-specific settings save

    pub const REBOOT: u16 = 68; //in message reboot settings

    // Use BUILD_INFO instead
    // DEPRECATED - pub const BF_BUILD_INFO              :u8 =69 //out message build date as well as some space for future expansion

    pub const DATAFLASH_SUMMARY: u16 = 70; //out message - get description of dataflash chip
    pub const DATAFLASH_READ: u16 = 71; //out message - get content of dataflash chip
    pub const DATAFLASH_ERASE: u16 = 72; //in message - erase dataflash chip

    // No-longer needed
    // DEPRECATED - pub const LOOP_TIME                  :u8 =73; //out message         Returns FC cycle time i.e loop time parameter // DEPRECATED
    // DEPRECATED - pub const SET_LOOP_TIME              :u8 =74; //in message          Sets FC cycle time i.e loop time parameter    // DEPRECATED

    pub const FAILSAFE_CONFIG: u16 = 75; //out message         Returns FC Fail-Safe settings
    pub const SET_FAILSAFE_CONFIG: u16 = 76; //in message          Sets FC Fail-Safe settings

    pub const RXFAIL_CONFIG: u16 = 77; //out message         Returns RXFAIL settings
    pub const SET_RXFAIL_CONFIG: u16 = 78; //in message          Sets RXFAIL settings

    pub const SDCARD_SUMMARY: u16 = 79; //out message         Get the state of the SD card

    pub const BLACKBOX_CONFIG: u16 = 80; //out message         Get blackbox settings
    pub const SET_BLACKBOX_CONFIG: u16 = 81; //in message          Set blackbox settings

    pub const TRANSPONDER_CONFIG: u16 = 82; //out message         Get transponder settings
    pub const SET_TRANSPONDER_CONFIG: u16 = 83; //in message          Set transponder settings

    pub const OSD_CONFIG: u16 = 84; //out message         Get osd settings - betaflight
    pub const SET_OSD_CONFIG: u16 = 85; //in message          Set osd settings - betaflight

    pub const OSD_CHAR_READ: u16 = 86; //out message         Get osd settings - betaflight
    pub const OSD_CHAR_WRITE: u16 = 87; //in message          Set osd settings - betaflight

    pub const VTX_CONFIG: u16 = 88; //out message         Get vtx settings - betaflight
    pub const SET_VTX_CONFIG: u16 = 89; //in message          Set vtx settings - betaflight

    // Betaflight Additional Commands
    pub const ADVANCED_CONFIG: u16 = 90;
    pub const SET_ADVANCED_CONFIG: u16 = 91;

    pub const FILTER_CONFIG: u16 = 92;
    pub const SET_FILTER_CONFIG: u16 = 93;

    pub const PID_ADVANCED: u16 = 94;
    pub const SET_PID_ADVANCED: u16 = 95;

    pub const SENSOR_CONFIG: u16 = 96;
    pub const SET_SENSOR_CONFIG: u16 = 97;

    pub const CAMERA_CONTROL: u16 = 98;

    pub const SET_ARMING_DISABLED: u16 = 99;

    //
    // OSD specific
    //
    pub const OSD_VIDEO_CONFIG: u16 = 180;
    pub const SET_OSD_VIDEO_CONFIG: u16 = 181;

    // External OSD displayport mode messages
    pub const DISPLAYPORT: u16 = 182;

    pub const COPY_PROFILE: u16 = 183;

    pub const BEEPER_CONFIG: u16 = 184;
    pub const SET_BEEPER_CONFIG: u16 = 185;

    pub const SET_TX_INFO: u16 = 186; // in message           Used to send runtime information from TX lua scripts to the firmware
    pub const TX_INFO: u16 = 187; // out message          Used by TX lua scripts to read information from the firmware

    pub const SET_OSD_CANVAS: u16 = 188; // in message           Set osd canvas size COLSxROWS
    pub const OSD_CANVAS: u16 = 189; // out message          Get osd canvas size COLSxROWS

    //
    // Multiwii original MSP commands
    //

    // See API_VERSION and MIXER_CONFIG
    //DEPRECATED - pub const IDENT               :u8 =100;    //out message         mixerMode + multiwii version + protocol version + capability variable

    pub const STATUS: u16 = 101; //out message         cycle time & errors_count & sensor present & box activation & current setting number
    pub const RAW_IMU: u16 = 102; //out message         9 DOF
    pub const SERVO: u16 = 103; //out message         servos
    pub const MOTOR: u16 = 104; //out message         motors
    pub const RC: u16 = 105; //out message         rc channels and more
    pub const RAW_GPS: u16 = 106; //out message         fix; number of satellites; lat; lon; alt; speed; ground course
    pub const COMP_GPS: u16 = 107; //out message         distance home; direction home
    pub const ATTITUDE: u16 = 108; //out message         2 angles:u8 =1 heading
    pub const ALTITUDE: u16 = 109; //out message         altitude; variometer
    pub const ANALOG: u16 = 110; // out message        vbat; power meter sum; rssi if available on RX
    pub const RC_TUNING: u16 = 111; //out message         rc rate; rc expo; roll pitch rate; yaw rate; dyn throttle PID
    pub const PID: u16 = 112; //out message         P I D gains (9 are used currently)
    // Legacy MultiWi command that was never used.
    //DEPRECATED - pub const BOX                 :u8 =113;    //out message         BOX setup (number is dependant of your setup)
    // Legacy command that was under constant change due to the naming vagueness; avoid at all costs - use more specific commands instead.
    //DEPRECATED - pub const MISC                :u8 =114;    //out message         power meter trig
    // Legacy MultiWi command that was never used and always wrong
    //DEPRECATED - pub const MOTOR_PINS          :u8 =115;    //out message         which pins are in use for motors & servos; for GUI
    pub const BOX_NAMES: u16 = 116; //out message         the aux switch names
    pub const PIDNAMES: u16 = 117; //out message         the PID names
    pub const WP: u16 = 118; //out message         get a WP; WP# is in the payload; returns (WP#; lat; lon; alt; flags) WP#0-home; WP#16-position hold
    pub const BOX_IDS: u16 = 119; //out message         get the permanent IDs associated to BOXes
    pub const SERVO_CONFIGURATIONS: u16 = 120; //out message         All servo configurations.
    pub const NAV_STATUS: u16 = 121; //out message         Returns navigation status
    pub const NAV_CONFIG: u16 = 122; //out message         Returns navigation parameters
    pub const MOTOR_3D_CONFIG: u16 = 124; //out message         Settings needed for reversible ESCs
    pub const RC_DEADBAND: u16 = 125; //out message         deadband for yaw alt pitch roll
    pub const SENSOR_ALIGNMENT: u16 = 126; //out message         orientation of acc;gyro;mag
    pub const LED_STRIP_MODE_COLOR: u16 = 127; //out message         Get LED strip mode_color settings
    pub const VOLTAGE_METERS: u16 = 128; //out message         Voltage (per meter)
    pub const CURRENT_METERS: u16 = 129; //out message         Amperage (per meter)
    pub const BATTERY_STATE: u16 = 130; //out message         Connected/Disconnected; Voltage; Current Used
    pub const MOTOR_CONFIG: u16 = 131; //out message         Motor configuration (min/max throttle; etc)
    pub const GPS_CONFIG: u16 = 132; //out message         GPS configuration
    pub const COMPASS_CONFIG: u16 = 133; //out message         Compass configuration
    pub const ESC_SENSOR_DATA: u16 = 134; //out message         Extra ESC data from 32-Bit ESCs (Temperature; RPM)
    pub const GPS_RESCUE: u16 = 135; //out message         GPS Rescue angle; returnAltitude; descentDistance; groundSpeed; sanityChecks and minSats
    pub const GPS_RESCUE_PIDS: u16 = 136; //out message         GPS Rescue throttleP and velocity PIDS + yaw P
    pub const VTX_TABLE_BAND: u16 = 137; //out message         vtxTable band/channel data
    pub const VTX_TABLE_POWER_LEVEL: u16 = 138; //out message         vtxTable powerLevel data
    pub const MOTOR_TELEMETRY: u16 = 139; //out message         Per-motor telemetry data (RPM; packet stats; ESC temp; etc.)

    pub const SIMPLIFIED_TUNING: u16 = 140; //out message    Simplified tuning values and enabled state
    pub const SET_SIMPLIFIED_TUNING: u16 = 141; //in message     Set simplified tuning positions and apply the calculated tuning
    pub const CALCULATE_SIMPLIFIED_PID: u16 = 142; //out message    Requests calculations of PID values based on sliders. Sends the calculated values back. But don't save anything to the firmware
    pub const CALCULATE_SIMPLIFIED_GYRO: u16 = 143; //out message    Requests calculations of gyro filter values based on sliders. Sends the calculated values back. But don't save anything to the firmware
    pub const CALCULATE_SIMPLIFIED_DTERM: u16 = 144; //out message    Requests calculations of gyro filter values based on sliders. Sends the calculated values back. But don't save anything to the firmware
    pub const VALIDATE_SIMPLIFIED_TUNING: u16 = 145; //out message    Returns an array of true/false showing which simplified tuning groups are matching with value and which are not

    pub const SET_RAW_RC: u16 = 200; //in message          8 rc chan
    pub const SET_RAW_GPS: u16 = 201; //in message          fix; number of satellites; lat; lon; alt; speed
    pub const SET_PID: u16 = 202; //in message          P I D gains (9 are used currently)
    // Legacy multiwii command that was never used.
    //DEPRECATED - pub const SET_BOX             :u8 =203;    //in message          BOX setup (number is dependant of your setup)
    pub const SET_RC_TUNING: u16 = 204; //in message          rc rate; rc expo; roll pitch rate; yaw rate; dyn throttle PID; yaw expo
    pub const ACC_CALIBRATION: u16 = 205; //in message          no param
    pub const MAG_CALIBRATION: u16 = 206; //in message          no param
    // Legacy command that was under constant change due to the naming vagueness; avoid at all costs - use more specific commands instead.
    //DEPRECATED - pub const SET_MISC            :u8 =207;    //in message          power meter trig + 8 free for future use
    pub const RESET_CONF: u16 = 208; //in message          no param
    pub const SET_WP: u16 = 209; //in message          sets a given WP (WP#;lat; lon; alt; flags)
    pub const SELECT_SETTING: u16 = 210; //in message          Select Setting Number (0-2)
    pub const SET_HEADING: u16 = 211; //in message          define a new heading hold direction
    pub const SET_SERVO_CONFIGURATION: u16 = 212; //in message          Servo settings
    pub const SET_MOTOR: u16 = 214; //in message          PropBalance function
    pub const SET_NAV_CONFIG: u16 = 215; //in message          Sets nav config parameters - write to the eeprom
    pub const SET_MOTOR_3D_CONFIG: u16 = 217; //in message          Settings needed for reversible ESCs
    pub const SET_RC_DEADBAND: u16 = 218; //in message          deadbands for yaw alt pitch roll
    pub const SET_RESET_CURR_PID: u16 = 219; //in message          resetting the current pid profile to defaults
    pub const SET_SENSOR_ALIGNMENT: u16 = 220; //in message          set the orientation of the acc;gyro;mag
    pub const SET_LED_STRIP_MODE_COLOR: u16 = 221; //in  message         Set LED strip mode_color settings
    pub const SET_MOTOR_CONFIG: u16 = 222; //out message         Motor configuration (min/max throttle; etc)
    pub const SET_GPS_CONFIG: u16 = 223; //out message         GPS configuration
    pub const SET_COMPASS_CONFIG: u16 = 224; //out message         Compass configuration
    pub const SET_GPS_RESCUE: u16 = 225; //in message          GPS Rescue angle; returnAltitude; descentDistance; groundSpeed and sanityChecks
    pub const SET_GPS_RESCUE_PIDS: u16 = 226; //in message          GPS Rescue throttleP and velocity PIDS + yaw P
    pub const SET_VTX_TABLE_BAND: u16 = 227; //in message          set vtxTable band/channel data (one band at a time)
    pub const SET_VTX_TABLE_POWER_LEVEL: u16 = 228; //in message          set vtxTable powerLevel data (one powerLevel at a time)

    // pub const BIND                :u8 =240;    //in message          no param
    // pub const ALARMS              :u8 =242;
    pub const SET_PASSTHROUGH: u16 = 245; // in message         Sets up passthrough to different peripherals (4way interface; uart; etc...)

    pub const EEPROM_WRITE: u16 = 250; //in message          no param
    pub const RESERVE_1: u16 = 251; //reserved for system usage
    pub const RESERVE_2: u16 = 252; //reserved for system usage
    pub const DEBUG_MSG: u16 = 253; //out message         debug string buffer
    pub const DEBUG: u16 = 254; //out message         debug1;debug2;debug3;debug4
    pub const V2_FRAME: u16 = 255; //MSPv2 payload indicator

    // Additional commands that are not compatible with MultiWii
    pub const STATUS_EX: u16 = 150; //out message         cycle time; errors_count; CPU load; CPU temperature; sensor present etc
    pub const UID: u16 = 160; //out message         Unique device ID
    pub const GPS_SV_INFO: u16 = 164; //out message         get Signal Strength (only U-Blox)
    pub const GPS_STATISTICS: u16 = 166; //out message         get GPS debugging data
    pub const MULTIPLE_MSP: u16 = 230; //out message         request multiple MSPs in one request - limit is the TX buffer; returns each MSP in the order they were requested starting with length of MSP; MSPs with input arguments are not supported
    pub const MODE_RANGES_EXTRA: u16 = 238; //out message         Reads the extra mode range data
    pub const ACC_TRIM: u16 = 240; //out message         get acc angle trim values
    pub const SET_ACC_TRIM: u16 = 239; //in message          set acc angle trim values
    pub const SERVO_MIX_RULES: u16 = 241; //out message         Returns servo mixer configuration
    pub const SET_SERVO_MIX_RULE: u16 = 242; //in message          Sets servo mixer configuration
    pub const SET_RTC: u16 = 246; //in message          Sets the RTC clock
    pub const RTC: u16 = 247; //out message         Gets the RTC clock
    pub const SET_BOARD_INFO: u16 = 248; //in message          Sets the board information for this board
    pub const SET_SIGNATURE: u16 = 249; //in message          Sets the signature of the board and serial number

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
