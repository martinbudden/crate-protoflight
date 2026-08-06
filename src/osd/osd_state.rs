#![cfg(feature = "osd")]

use radio_controllers::RcMode;

use crate::{
    display::{
        Display, DisplayPort, DisplayPortDeviceType,
        DisplayPortLayer::{self, Background},
    },
    osd::{OsdConfig, OsdDrawContext, elements::OsdElements},
};

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum OsdState {
    #[default]
    Init,
    Idle,
    Start,
    ProcessStats1,
    RefreshStats,
    ProcessStats2,
    ProcessStats3,
    UpdateAlarms,
    RefreshPreArm,
    UpdateCanvas,
    SetCurrentElement {
        element_index: usize,
    },
    // Elements are handled in two steps, drawing into a buffer, and then sending to the display
    DrawCurrentElementStep {
        element_index: usize,
    },
    DisplayCurrentElementStep {
        element_index: usize,
        more_to_draw: bool,
    },
    UpdateHeartbeat,
    Commit,
    Transfer,
}

#[allow(unused)]
impl OsdState {
    /// Triggers a canvas refresh sync for blinking animations.
    fn sync_blink(&mut self, _time_microseconds: u32) {
        _ = self;
    }

    /// Renders initial assets, such as boot logos, on startup.
    fn draw_logo_and_complete_initialization(&mut self) {
        _ = self;
    }

    /// Evaluates phase 1 display statistics data blocks.
    fn process_stats1(&mut self, _time_us: u32) -> bool {
        _ = self;
        false
    }

    /// Evaluates phase 2 display statistics data blocks.
    fn process_stats2(&mut self, _time_us: u32) {
        _ = self;
    }

    /// Evaluates phase 3 display statistics data blocks.
    fn process_stats3(&mut self) {
        _ = self;
    }

    /// Instructs the canvas to refresh specific statistics modules.
    fn refresh_stats(&mut self) -> bool {
        _ = self;
        false
    }

    /// Iterates through and processes queued system threshold warning logs.
    fn update_alarms(&mut self) {
        _ = self;
    }
}

#[allow(unused)]
impl OsdState {
    pub fn start(&mut self) -> bool {
        if *self == OsdState::Idle {
            *self = OsdState::Start;
            true
        } else {
            false
        }
    }

    #[allow(clippy::too_many_lines)]
    pub async fn update_display_iteration<D: Display>(
        &mut self,
        osd_elements: &mut OsdElements,
        draw_ctx: &mut OsdDrawContext,
        display_port: &mut D,
        osd_config: &OsdConfig,
        time_us: u32,
    ) {
        *self = match core::mem::take(self) {
            Self::Init => {
                if display_port.check_ready(false) {
                    display_port.begin_transaction(DisplayPort::DISPLAY_TRANSACTION_OPTION_RESET_DRAWING);
                    self.draw_logo_and_complete_initialization();
                    Self::Commit
                } else {
                    // Frsky OSD needs a display redraw after search for MAX7456 devices
                    if display_port.device_type() == DisplayPortDeviceType::FrskyOsd {
                        display_port.redraw();
                    }
                    Self::Init
                }
            }
            Self::Start => {
                // don't touch buffers if DMA transaction is in progress
                if display_port.is_transfer_in_progress() { Self::Start } else { Self::UpdateHeartbeat }
            }
            Self::UpdateHeartbeat => {
                if display_port.heartbeat() != 0 {
                    // Extraordinary action was taken, so return without allowing state_duration_fraction_us table to be updated
                    return;
                }
                Self::ProcessStats1
            }
            Self::ProcessStats1 => {
                // transaction begins here since RefreshStats draws to the screen
                display_port.begin_transaction(DisplayPort::DISPLAY_TRANSACTION_OPTION_RESET_DRAWING);
                if self.process_stats1(time_us) { Self::RefreshStats } else { Self::ProcessStats2 }
            }
            Self::RefreshStats => {
                if self.refresh_stats() {
                    // draws the statistics to the screen
                    Self::ProcessStats2
                } else {
                    Self::RefreshStats
                }
            }
            Self::ProcessStats2 => {
                self.process_stats2(time_us); // may clear screen
                Self::ProcessStats3
            }
            Self::ProcessStats3 => {
                self.process_stats3();
                #[cfg(feature = "cms")]
                if display_port.is_grabbed() {
                    Self::Commit
                }
                Self::UpdateAlarms
            }
            Self::UpdateAlarms => {
                self.update_alarms();
                //if osd.resume_refresh_at_us == 0 { Self::UpdateCanvas } else { Self::Transfer }
                Self::UpdateCanvas
            }
            Self::UpdateCanvas => {
                if draw_ctx.rx_message.rc_modes.test(RcMode::OSD) {
                    // Hide OSD when OSD SW mode is active
                    display_port.clear_screen().await;
                    Self::Commit
                } else {
                    if display_port.layer_supported(Background) {
                        // Background layer is supported, overlay it onto the foreground
                        // so that we only need to draw the active parts of the elements.
                        display_port.layer_copy(DisplayPortLayer::Foreground, DisplayPortLayer::Background);
                    } else {
                        // Background layer not supported, just clear the foreground in preparation
                        // for drawing the elements including their backgrounds.
                        display_port.clear_screen().await;
                    }
                    self.sync_blink(time_us);
                    // update the orientation, so it is only needed to be calculated once for all elements that require it
                    let orientation = draw_ctx.orientation;
                    osd_elements.update_cache(
                        orientation.calculate_roll_degrees(),
                        orientation.calculate_pitch_degrees(),
                        orientation.calculate_yaw_degrees(),
                    );
                    Self::SetCurrentElement { element_index: 0 }
                }
            }
            Self::SetCurrentElement { element_index } => {
                if osd_elements.set_current_element_by_index(element_index) {
                    Self::DrawCurrentElementStep { element_index }
                } else {
                    // We've exhausted all the elements, so move on to the next state.
                    /* if ctx.cockpit.is_armed() && self.config.osd_show_spec_prearm {
                        Self::RefreshPreArm
                    } else {
                        Self::Commit
                    };*/
                    Self::Commit
                }
            }
            Self::DrawCurrentElementStep { element_index } => {
                // Drawing an element renders it to the element buffer
                // For complex elements (like the artificial horizon) this may take several steps.

                //let active_element_index = osd_elements.active_element_index();*/
                let more_to_draw = osd_elements.draw_current_element(draw_ctx, display_port, osd_config).await;

                // Display the part of the element we have drawn.
                Self::DisplayCurrentElementStep { element_index, more_to_draw }
            }
            // DisplayElementStep copies the element buffer to the displayport buffer
            Self::DisplayCurrentElementStep { element_index, more_to_draw } => {
                let more_to_display = osd_elements.display_current_element(display_port);
                if more_to_display {
                    // this element requires several steps display it , so display the next step
                    Self::DisplayCurrentElementStep { element_index, more_to_draw }
                } else {
                    // if the element needs more draw steps, the do those, otherwise move onto the next element
                    if more_to_draw {
                        Self::DrawCurrentElementStep { element_index }
                    } else {
                        Self::SetCurrentElement { element_index: element_index + 1 }
                    }
                }
            }
            Self::RefreshPreArm => {
                if osd_elements.draw_spec() {
                    // Rendering is complete
                    Self::Commit
                } else {
                    Self::RefreshPreArm
                }
            }
            Self::Commit => {
                display_port.commit_transaction();
                //if osd.resume_refresh_at_us == 0 { Self::Transfer } else { Self::Idle }
                Self::Transfer
            }
            Self::Transfer => {
                // Transfer the display port buffer to the actual display port hardware
                match display_port.transfer_screen().await {
                    Ok(still_transferring) => {
                        if still_transferring {
                            // The transfer is not complete, so continue transferring
                            Self::Transfer
                        } else {
                            Self::Idle
                        }
                    }
                    Err(_err) => {
                        // If there has been an error, eg an SPI bus or hardware fault, then just ignore it.
                        Self::Idle
                    }
                }
            }
            Self::Idle => Self::Idle,
        }
    }
}
