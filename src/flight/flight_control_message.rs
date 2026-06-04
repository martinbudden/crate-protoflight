#[allow(unused)]
use radio_controllers::{Rates, RcModes, RcSticks, RxFrame};
use simple_bitset::BitSet64;

/// Message for communicating a radio control command between tasks.<br><br>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct FlightControlMessage {
    pub rc_modes: BitSet64,
    pub tick_count: u32,
    pub throttle_stick: f32,
    pub roll_stick_dps: f32,
    pub pitch_stick_dps: f32,
    pub yaw_stick_dps: f32,
    pub roll_stick_degrees: f32,
    pub pitch_stick_degrees: f32,
    pub stabilization_mode: u8,
    pub failsafe: u8,
}
const _: () = assert!(core::mem::size_of::<FlightControlMessage>() == 40);

impl Default for FlightControlMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl FlightControlMessage {
    pub const fn new() -> Self {
        Self {
            rc_modes: BitSet64::new(),
            tick_count: 0,
            throttle_stick: 0.0,
            roll_stick_dps: 0.0,
            pitch_stick_dps: 0.0,
            yaw_stick_dps: 0.0,
            roll_stick_degrees: 0.0,
            pitch_stick_degrees: 0.0,
            stabilization_mode: 0,
            failsafe: 0,
        }
    }
}

impl FlightControlMessage {
    /// Create a `RadioControlMessage` from an `RxFrame`, applying rates and including `RcModes`.
    pub fn new_from(
        rx_frame: &RxFrame,
        rates: &Rates,
        rc_modes: &RcModes,
        tick_count: u32,
        failsafe: u8,
    ) -> FlightControlMessage {
        // get the stick values from the rx_frame.
        let sticks = RcSticks::from(*rx_frame);

        // apply rates to the stick values.
        let roll_stick_dps = rates.apply(Rates::ROLL, sticks.roll);
        let pitch_stick_dps = rates.apply(Rates::PITCH, sticks.pitch);
        let yaw_stick_dps = rates.apply(Rates::YAW, sticks.yaw);

        // scale the stick angles.
        let roll_stick_degrees = sticks.roll * rates.max_roll_angle_degrees;
        let pitch_stick_degrees = sticks.pitch * rates.max_pitch_angle_degrees;

        // Get the rc_modes (eg altitude hold, gps home) (used by the autopilot),
        // and the stabilization mode (eg STABILIZATION_MODE_RATE) (used by the flight controller).
        let (rc_modes, stabilization_mode) = rc_modes.update_modes();

        FlightControlMessage {
            rc_modes,
            tick_count,

            throttle_stick: sticks.throttle,
            roll_stick_dps,
            pitch_stick_dps,
            yaw_stick_dps,
            roll_stick_degrees,
            pitch_stick_degrees,

            stabilization_mode,
            failsafe,
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
        is_full::<FlightControlMessage>();
    }
    #[test]
    fn sizeof() {
        assert_eq!(40, core::mem::size_of::<FlightControlMessage>());
    }
}
