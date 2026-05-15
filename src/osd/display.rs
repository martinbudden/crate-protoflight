#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Osd {}

impl Osd {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for Osd {
    fn default() -> Self {
        Self::new()
    }
}

impl Osd {
    pub const PROFILE_COUNT: usize = 2;
    pub const PROFILE_NAME_LENGTH: usize = 16;
    pub const RC_CHANNELS_COUNT: usize = 4;
    pub const TIMER_COUNT: usize = 2;

    pub const _LOGO_ROW_COUNT: usize = 4;
    pub const _LOGO_COLUMN_COUNT: usize = 24;

    pub const SD_ROWS: u8 = 16;
    pub const SD_COLS: u8 = 30;
    pub const _HD_ROWS: u8 = 20;
    pub const _HD_COLS: u8 = 53;

    pub const FRAMERATE_DEFAULT_HZ: u16 = 12;

    pub const ESC_RPM_ALARM_OFF: i16 = -1;
    pub const ESC_CURRENT_ALARM_OFF: i16 = -1;
    pub const ESC_TEMPERATURE_ALARM_OFF: u8 = 0;

    pub const UNITS_METRIC: u8 = 0;
    pub const _UNITS_IMPERIAL: u8 = 1;

    pub const LOGO_ARMING_OFF: u8 = 0;
    pub const _LOGO_ARMING_ON: u8 = 1;
    pub const _LOGO_ARMING_FIRST: u8 = 2;
}

impl Osd {
    // TODO: placeholder OSD update display
    #[allow(clippy::unused_self)]
    pub fn update_display(self) {}
}
