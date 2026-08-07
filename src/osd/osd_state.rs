#![cfg(feature = "osd")]

use radio_controllers::RcMode;

use crate::{
    display::{
        Display, DisplayPort, DisplayPortDeviceType,
        DisplayPortLayer::{self, Background},
    },
    osd::{OsdConfig, OsdDrawContext, elements::OsdElements},
    tasks::init::DisplayPortMutex,
};

/*
Idle
 │
 ▼
Start
 │
 ▼
Heartbeat
 │
 ▼
Stats
 │
 ▼
Canvas
 │
 ▼
Element 0
 │
 ▼
Element 1
 │
 ▼
...
 │
 ▼
Commit
 │
 ▼
Transfer
 │
 ▼
Idle
*/
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
    RenderCurrentElement {
        element_index: usize,
    },
    FlushCurrentElement {
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

    async fn init(&mut self, display_port_mutex: &'static DisplayPortMutex) -> Self {
        let mut display_port = display_port_mutex.lock().await;
        if display_port.check_ready(false) {
            display_port.begin_transaction(DisplayPort::DISPLAY_TRANSACTION_OPTION_RESET_DRAWING);
            self.draw_logo_and_complete_initialization();
            return Self::Commit;
        }
        // Frsky OSD needs a display redraw after search for MAX7456 devices
        if display_port.device_type() == DisplayPortDeviceType::FrskyOsd {
            display_port.redraw();
        }
        Self::Init
    }

    async fn start(&mut self, display_port_mutex: &'static DisplayPortMutex) -> Self {
        // don't touch buffers if DMA transaction is in progress
        let mut display_port = display_port_mutex.lock().await;
        if display_port.is_transfer_in_progress() { Self::Start } else { Self::UpdateHeartbeat }
    }

    async fn update_heartbeat(&mut self, display_port_mutex: &'static DisplayPortMutex) -> Self {
        let mut display_port = display_port_mutex.lock().await;
        if display_port.heartbeat() == 0 {
            Self::ProcessStats1
        } else {
            // Extraordinary action was taken, so return without allowing state_duration_fraction_us table to be updated
            Self::UpdateHeartbeat
        }
    }

    /// Evaluates phase 1 display statistics data blocks.
    async fn process_stats1(&mut self, display_port_mutex: &'static DisplayPortMutex, _time_us: u32) -> Self {
        // transaction begins here since RefreshStats draws to the screen
        let mut display_port = display_port_mutex.lock().await;
        display_port.begin_transaction(DisplayPort::DISPLAY_TRANSACTION_OPTION_RESET_DRAWING);
        // { Self::RefreshStats } else { Self::ProcessStats2 }
        Self::RefreshStats
    }

    /// Instructs the canvas to refresh specific statistics modules.
    fn refresh_stats(&mut self) -> Self {
        _ = self;
        Self::ProcessStats2
        /*if self.refresh_stats() {
            Self::ProcessStats2
        } else {
            Self::RefreshStats
        }*/
    }
    /// Evaluates phase 2 display statistics data blocks.
    fn process_stats2(&mut self, _time_us: u32) -> Self {
        _ = self;
        Self::ProcessStats3
    }

    /// Evaluates phase 3 display statistics data blocks.
    fn process_stats3(&mut self) -> Self {
        _ = self;
        Self::UpdateAlarms
    }

    /// Iterates through and processes queued system threshold warning logs.
    fn update_alarms(&mut self) -> Self {
        _ = self;
        //if osd.resume_refresh_at_us == 0 { Self::UpdateCanvas } else { Self::Transfer }
        Self::UpdateCanvas
    }

    async fn update_canvas(
        &mut self,
        osd_elements: &mut OsdElements,
        draw_ctx: &OsdDrawContext,
        display_port_mutex: &'static DisplayPortMutex,
        osd_config: &OsdConfig,
        time_us: u32,
    ) -> Self {
        let mut display_port = display_port_mutex.lock().await;
        if draw_ctx.rx_message.rc_modes.test(RcMode::OSD) {
            // Hide OSD when OSD SW mode is active
            display_port.clear_screen().await;
            return Self::Commit;
        }
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

    fn set_current_element(element_index: usize, osd_elements: &mut OsdElements) -> Self {
        if osd_elements.set_current_element_by_index(element_index) {
            Self::RenderCurrentElement { element_index }
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

    async fn render_current_element(
        element_index: usize,
        osd_elements: &mut OsdElements,
        draw_ctx: &OsdDrawContext,
        display_port_mutex: &'static DisplayPortMutex,
        osd_config: &OsdConfig,
    ) -> Self {
        // Render the current element to the element buffer
        // For complex elements (like the artificial horizon) this may take several steps.

        let mut display_port = display_port_mutex.lock().await;
        let more_to_draw = osd_elements.draw_current_element(draw_ctx, &mut *display_port, osd_config).await;

        // Flush the part of the element we have rendered.
        Self::FlushCurrentElement { element_index, more_to_draw }
    }

    async fn flush_current_element(
        element_index: usize,
        more_to_draw: bool,
        osd_elements: &mut OsdElements,
        display_port_mutex: &'static DisplayPortMutex,
    ) -> Self {
        let mut display_port = display_port_mutex.lock().await;
        let more_to_display = osd_elements.display_current_element(&mut *display_port);
        if more_to_display {
            // this element requires several steps display it , so display the next step
            return Self::FlushCurrentElement { element_index, more_to_draw };
        }
        // if the element needs more draw steps, the do those, otherwise move onto the next element
        if more_to_draw {
            Self::RenderCurrentElement { element_index }
        } else {
            Self::SetCurrentElement { element_index: element_index + 1 }
        }
    }

    fn refresh_prearm(osd_elements: &mut OsdElements) -> Self {
        if osd_elements.draw_spec() {
            // Rendering is complete
            Self::Commit
        } else {
            Self::RefreshPreArm
        }
    }
    async fn commit(display_port_mutex: &'static DisplayPortMutex) -> Self {
        let mut display_port = display_port_mutex.lock().await;
        display_port.commit_transaction();
        //if osd.resume_refresh_at_us == 0 { Self::Transfer } else { Self::Idle }
        Self::Transfer
    }
    async fn transfer(display_port_mutex: &'static DisplayPortMutex) -> Self {
        // Transfer the display port buffer to the actual display port hardware
        let mut display_port = display_port_mutex.lock().await;
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
}

#[allow(unused)]
impl OsdState {
    pub fn start_frame(&mut self) -> bool {
        if *self == OsdState::Idle {
            *self = OsdState::Start;
            true
        } else {
            false
        }
    }

    pub async fn update_display_iteration(
        &mut self,
        osd_elements: &mut OsdElements,
        draw_ctx: &OsdDrawContext,
        display_port_mutex: &'static DisplayPortMutex,
        osd_config: &OsdConfig,
        time_us: u32,
    ) {
        *self = match core::mem::take(self) {
            Self::Init => self.init(display_port_mutex).await,
            Self::Start => self.start(display_port_mutex).await,
            Self::UpdateHeartbeat => self.update_heartbeat(display_port_mutex).await,
            Self::ProcessStats1 => self.process_stats1(display_port_mutex, time_us).await,
            Self::RefreshStats => self.refresh_stats(),
            Self::ProcessStats2 => self.process_stats2(time_us), // may clear screen
            Self::ProcessStats3 => self.process_stats3(),
            Self::UpdateAlarms => self.update_alarms(),
            Self::UpdateCanvas => {
                self.update_canvas(osd_elements, draw_ctx, display_port_mutex, osd_config, time_us).await
            }
            Self::SetCurrentElement { element_index } => Self::set_current_element(element_index, osd_elements),
            Self::RenderCurrentElement { element_index } => {
                Self::render_current_element(element_index, osd_elements, draw_ctx, display_port_mutex, osd_config)
                    .await
            }
            // FlushCurrentElement copies the element buffer to the displayport buffer
            Self::FlushCurrentElement { element_index, more_to_draw } => {
                Self::flush_current_element(element_index, more_to_draw, osd_elements, display_port_mutex).await
            }
            Self::RefreshPreArm => Self::refresh_prearm(osd_elements),
            Self::Commit => Self::commit(display_port_mutex).await,
            Self::Transfer => Self::transfer(display_port_mutex).await,
            Self::Idle => Self::Idle,
        }
    }
}
