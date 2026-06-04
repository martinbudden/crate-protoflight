use crate::flight::{flight_control_message::FlightControlMessage, vehicle_controller::VehicleController};
use vqm::{Quaternionf32, Vector3df32, Vector4df32};

#[allow(unused)]
pub trait VehicleControl {
    fn vehicle_controller(&self) -> &VehicleController;
    fn vehicle_controller_mut(&mut self) -> &mut VehicleController;

    fn calculate_motor_commands(
        &mut self,
        gyro_rps: Vector3df32,
        orientation: Quaternionf32,
        delta_t: f32,
        controls: FlightControlMessage,
    ) -> (Vector4df32, bool);
}
