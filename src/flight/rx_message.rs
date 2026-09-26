use radio_controllers::RxChannelsLinkStatus;
#[allow(unused)]
use radio_controllers::{Rates, RcModes, RcSticks, RxFrame, RxLinkStatus};
use simple_bitset::BitSet64;

/// Message for communicating a radio control command between tasks.<br><br>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct RcControls {
    pub tick_count: u32,
    pub throttle_stick: f32,
    pub roll_stick_dps: f32,
    pub pitch_stick_dps: f32,
    pub yaw_stick_dps: f32,
    pub roll_stick_degrees: f32,
    pub pitch_stick_degrees: f32,
    pub controls_pwm: [u16; 4],
    pub link_status: RxLinkStatus,
    pub rssi: u8,
}
const _: () = assert!(core::mem::size_of::<RcControls>() == 40);

impl Default for RcControls {
    fn default() -> Self {
        Self::new()
    }
}

impl RcControls {
    pub const fn new() -> Self {
        Self {
            tick_count: 0,
            throttle_stick: 0.0,
            roll_stick_dps: 0.0,
            pitch_stick_dps: 0.0,
            yaw_stick_dps: 0.0,
            roll_stick_degrees: 0.0,
            pitch_stick_degrees: 0.0,
            controls_pwm: [0u16; 4],
            link_status: RxLinkStatus::Ok,
            rssi: 0,
        }
    }
}

/// Message for communicating a radio control command between tasks.<br><br>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct RxMessage {
    pub rc_modes: BitSet64,
    pub rc_controls: RcControls,
}
const _: () = assert!(core::mem::size_of::<RxMessage>() == 48);

impl Default for RxMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl RxMessage {
    pub const fn new() -> Self {
        Self { rc_modes: BitSet64::new(), rc_controls: RcControls::new() }
    }
}

impl RxMessage {
    /// Create a `RadioControlMessage` from an `RxFrame`, applying rates and including `RcModes`.
    pub fn new_from(
        rx_channels_link_status: &RxChannelsLinkStatus,
        rates: &Rates,
        rc_modes: &RcModes,
        tick_count: u32,
    ) -> RxMessage {
        // get the stick values from the rx_frame.
        let sticks = RcSticks::from(rx_channels_link_status.channels);

        // apply rates to the stick values.
        let roll_stick_dps = rates.apply(Rates::ROLL, sticks.roll);
        let pitch_stick_dps = rates.apply(Rates::PITCH, sticks.pitch);
        let yaw_stick_dps = rates.apply(Rates::YAW, sticks.yaw);

        // scale the stick angles.
        let roll_stick_degrees = sticks.roll * rates.max_roll_angle_degrees;
        let pitch_stick_degrees = sticks.pitch * rates.max_pitch_angle_degrees;

        // Get the rc_modes (eg altitude hold, gps home) (used by the autopilot),
        // and the stabilization mode (eg STABILIZATION_MODE_RATE) (used by the flight controller).

        let controls_pwm = [
            rx_channels_link_status.channels[0],
            rx_channels_link_status.channels[1],
            rx_channels_link_status.channels[2],
            rx_channels_link_status.channels[3],
        ];
        let link_status = rx_channels_link_status.link_status;
        RxMessage {
            rc_modes: rc_modes.active_modes,
            rc_controls: RcControls {
                tick_count,

                throttle_stick: sticks.throttle,
                roll_stick_dps,
                pitch_stick_dps,
                yaw_stick_dps,
                roll_stick_degrees,
                pitch_stick_degrees,
                controls_pwm,

                link_status,
                rssi: 0,
            },
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
        is_full::<RxMessage>();
    }
    #[test]
    fn sizeof() {
        assert_eq!(48, core::mem::size_of::<RxMessage>());
    }
}
