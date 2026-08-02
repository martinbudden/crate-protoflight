use crate::flight::{rx_message::RcControls, vehicle_controller::VehicleController};
use simple_bitset::BitSet64;
use vqm::{Quaternionf32, Vector3f32, Vector4f32};

#[allow(unused)]
pub trait VehicleControl {
    fn vehicle_controller(&self) -> &VehicleController;
    fn vehicle_controller_mut(&mut self) -> &mut VehicleController;

    fn calculate_motor_commands(
        &mut self,
        gyro_rps: Vector3f32,
        orientation: Quaternionf32,
        delta_t: f32,
        controls: RcControls,
        rc_modes: BitSet64,
    ) -> (Vector4f32, bool);
}
