//#![allow(unused)]

use radio_controllers::{RcModes, RcMode};
use vqm::Quaternionf32;

use crate::{
    display::{Display, DisplayPort, DisplayPortDeviceType, DisplayPortLayer},
    flight::ArmingFlags,
    osd::{OsdConfig, elements::OsdElements},
    sensors::BatteryMessage,
};

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum OsdState {
    #[default]
    Init,
    Idle,
    Check,
    ProcessStats1,
    RefreshStats,
    ProcessStats2,
    ProcessStats3,
    UpdateAlarms,
    RefreshPreArm,
    UpdateCanvas,
    // Elements are handled in two steps, drawing into a buffer, and then sending to the display
    DrawElement,
    DisplayElement,
    UpdateHeartbeat,
    Commit,
    Transfer,
}

#[derive(Debug)]
pub struct OsdDrawContext<'a, D: Display> {
    // Accepts any type that implements the Display trait
    pub display_port: &'a mut D,
    pub orientation: Quaternionf32,
    pub arming_flags: ArmingFlags,
    #[cfg(feature = "battery")]
    pub battery_message: BatteryMessage,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Osd {
    /// The current operational state of the display loop.
    pub state: OsdState,

    /// Flag tracking whether more graphical elements are queued for rendering.
    pub more_elements_to_draw: bool,

    /// Tracks if the display device supports a dedicated background layer.
    pub background_layer_supported: bool,

    /// Timer/timestamp storage to delay or resume refreshing the canvas.
    pub resume_refresh_at_us: u32,

    /// Subsystem handling layout, tracking, and rendering of individual OSD items.
    pub elements: OsdElements,

    /// User and system configuration options for the OSD.
    pub config: OsdConfig,

    /// Cache mapping historical rendering durations per active element index.
    pub element_duration_fraction_us: [u32; 32],
}

impl Osd {
    pub const fn new() -> Self {
        Self {
            state: OsdState::Init,
            more_elements_to_draw: false,
            background_layer_supported: false,
            resume_refresh_at_us: 0,
            elements: OsdElements::new(),
            config: OsdConfig::new(),
            element_duration_fraction_us: [0u32; 32],
        }
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

#[allow(unused)]
#[allow(clippy::unused_self)]
impl Osd {
    /// Simulates a system uptime lookup tool in microseconds.
    fn time_us(&self) -> u32 {
        0
    }

    /// Triggers a canvas refresh sync for blinking animations.
    fn sync_blink(&mut self, _time_microseconds: u32) {}

    /// Renders initial assets, such as boot logos, on startup.
    fn draw_logo_and_complete_initialization(&mut self) {}

    /// Evaluates phase 1 display statistics data blocks.
    fn process_stats1(&mut self, _time_us: u32) -> bool {
        false
    }

    /// Evaluates phase 2 display statistics data blocks.
    fn process_stats2(&mut self, _time_us: u32) {}

    /// Evaluates phase 3 display statistics data blocks.
    fn process_stats3(&mut self) {}

    /// Instructs the canvas to refresh specific statistics modules.
    fn refresh_stats(&mut self) -> bool {
        false
    }

    /// Iterates through and processes queued system threshold warning logs.
    fn update_alarms(&mut self) {}
}

impl Osd {
    // TODO: placeholder OSD update display
    #[allow(clippy::unused_self)]
    pub async fn update_display<D: Display>(&mut self, draw_ctx: &mut OsdDrawContext<'_, D>, time_microseconds: u32) {
        /*if draw_ctx.display_port.is_grabbed() {
            return;
        }*/
        if self.state == OsdState::Idle {
            self.state = OsdState::Check;
        } else if self.state != OsdState::Init {
            return;
        }
        while self.state != OsdState::Idle {
            self.update_display_iteration(draw_ctx, time_microseconds).await;
        }
    }
}

impl Osd {
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::unused_self)]
    //pub async fn update_display_iteration<D: Display>(&mut self, draw_ctx: &mut OsdDrawContext<D>, time_microseconds: u32) {
    pub async fn update_display_iteration<D: Display>(
        &mut self,
        draw_ctx: &mut OsdDrawContext<'_, D>,
        time_microseconds: u32,
    ) {
        match self.state {
            OsdState::Init => {
                if !draw_ctx.display_port.check_ready(false) {
                    // Frsky OSD needs a display redraw after search for MAX7456 devices
                    if draw_ctx.display_port.device_type() == DisplayPortDeviceType::FrskyOsd {
                        draw_ctx.display_port.redraw();
                        return;
                    }
                }
                draw_ctx.display_port.begin_transaction(DisplayPort::DISPLAY_TRANSACTION_OPTION_RESET_DRAWING);
                self.draw_logo_and_complete_initialization();
                self.state = OsdState::Commit;
            }
            OsdState::Check => {
                // don't touch buffers if DMA transaction is in progress
                if draw_ctx.display_port.is_transfer_in_progress() {
                    return;
                }
                self.state = OsdState::UpdateHeartbeat;
            }
            OsdState::UpdateHeartbeat => {
                if draw_ctx.display_port.heartbeat() != 0 {
                    // Extraordinary action was taken, so return without allowing state_duration_fraction_us table to be updated
                    return;
                }
                self.state = OsdState::ProcessStats1;
            }
            OsdState::ProcessStats1 => {
                // transaction begins here since RefreshStats draws to the screen
                draw_ctx.display_port.begin_transaction(DisplayPort::DISPLAY_TRANSACTION_OPTION_RESET_DRAWING);
                self.state = if self.process_stats1(time_microseconds) {
                    OsdState::RefreshStats
                } else {
                    OsdState::ProcessStats2
                };
            }
            OsdState::RefreshStats => {
                if self.refresh_stats() {
                    // draws the statistics to the screen
                    self.state = OsdState::ProcessStats2;
                }
            }
            OsdState::ProcessStats2 => {
                self.process_stats2(time_microseconds); // may clear screen
                self.state = OsdState::ProcessStats3;
            }
            OsdState::ProcessStats3 => {
                self.process_stats3();
                #[cfg(feature = "cms")]
                if draw_ctx.display_port.is_grabbed() {
                    self.state = OsdState::Commit;
                }
                self.state = OsdState::UpdateAlarms;
            }
            OsdState::UpdateAlarms => {
                self.update_alarms();
                // Note:_state = _resume_refresh_at_us ? STATE_TRANSFER : STATE_UPDATE_CANVAS;
                self.state = OsdState::UpdateCanvas;
            }
            OsdState::UpdateCanvas => {
                let rc_modes = RcModes::new(); // TODO: use actual RC Modes, not this placeholder.
                if rc_modes.is_mode_active(RcMode::OSD) {
                    // Hide OSD when OSD SW mode is active
                    draw_ctx.display_port.clear_screen().await;
                    self.state = OsdState::Commit;
                    return;
                }
                if self.background_layer_supported {
                    // Background layer is supported, overlay it onto the foreground
                    // so that we only need to draw the active parts of the elements.
                    draw_ctx.display_port.layer_copy(DisplayPortLayer::Foreground, DisplayPortLayer::Background);
                } else {
                    // Background layer not supported, just clear the foreground in preparation
                    // for drawing the elements including their backgrounds.
                    draw_ctx.display_port.clear_screen().await;
                }
                self.sync_blink(time_microseconds);
                // update the orientation, so it is only needed to be done once for all elements that require it
                let orientation = draw_ctx.orientation;
                self.elements.update_attitude(
                    orientation.calculate_roll_degrees(),
                    orientation.calculate_pitch_degrees(),
                    orientation.calculate_yaw_degrees(),
                );
                self.state = OsdState::DrawElement;
            }
            OsdState::DrawElement => {
                const OSD_EXEC_TIME_SHIFT: u32 = 5;

                let active_element_index = self.elements.active_element_index();

                let start_element_time = self.time_us();
                self.more_elements_to_draw = self.elements.draw_next_active_element(draw_ctx);
                let execute_time_us = self.time_us() - start_element_time;

                if execute_time_us > (self.element_duration_fraction_us[active_element_index] >> OSD_EXEC_TIME_SHIFT) {
                    self.element_duration_fraction_us[active_element_index] = execute_time_us << OSD_EXEC_TIME_SHIFT;
                } else if self.element_duration_fraction_us[active_element_index] > 0 {
                    // Slowly decay the max time
                    self.element_duration_fraction_us[active_element_index] -= 1;
                }
                if self.elements.is_render_pending() {
                    // Render the element just drawn
                    self.state = OsdState::DisplayElement;
                    return;
                }
                if self.more_elements_to_draw {
                    return;
                }
                self.state = OsdState::Commit;
                /*self.state = if ctx.cockpit.is_armed() && self.config.osd_show_spec_prearm {
                    OsdState::RefreshPreArm
                } else {
                    OsdState::Commit
                };*/
            }
            OsdState::DisplayElement => {
                let more_to_display = self.elements.display_active_element(draw_ctx);
                if !more_to_display {
                    // finished displaying this element, so move on to the next one if there is one
                    if self.more_elements_to_draw {
                        self.state = OsdState::DrawElement;
                    } else {
                        /*self.state = if ctx.cockpit.is_armed() && self.config.osd_show_spec_prearm {
                            OsdState::RefreshPreArm
                        } else {
                            OsdState::Commit
                        };*/
                        self.state = OsdState::Commit;
                    }
                }
            }
            OsdState::RefreshPreArm => {
                if self.elements.draw_spec() {
                    // Rendering is complete
                    self.state = OsdState::Commit;
                }
            }
            OsdState::Commit => {
                draw_ctx.display_port.commit_transaction();
                self.state = if self.resume_refresh_at_us != 0 { OsdState::Idle } else { OsdState::Transfer };
            }
            OsdState::Transfer => {
                match draw_ctx.display_port.draw_screen().await {
                    Ok(screen_still_transferring) => {
                        if screen_still_transferring {
                            return; // DMA buffer filled up, keep streaming next iteration
                        }
                        self.state = OsdState::Idle;
                    }
                    Err(_err) => {
                        // Handle SPI bus or hardware faults gracefully
                        self.state = OsdState::Idle;
                    }
                }
            }
            OsdState::Idle => {
                self.state = OsdState::Check;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<Osd>();
        is_full::<OsdState>();
        //is_normal::<OsdDrawContext>();
    }
}
