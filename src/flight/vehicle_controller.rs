#[allow(unused)]
pub trait VehicleControlInitializing {
    fn sensor_fusion_filter_is_initializing(&self) -> bool;
    fn set_sensor_fusion_filter_is_initializing(&mut self, sensor_fusion_filter_is_initializing: bool);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleController {
    pub(crate) sensor_fusion_filter_is_initializing: bool,
}

impl VehicleController {
    pub const fn new() -> Self {
        Self { sensor_fusion_filter_is_initializing: false }
    }
}

impl Default for VehicleController {
    fn default() -> Self {
        Self::new()
    }
}
impl VehicleControlInitializing for VehicleController {
    fn sensor_fusion_filter_is_initializing(&self) -> bool {
        self.sensor_fusion_filter_is_initializing
    }
    fn set_sensor_fusion_filter_is_initializing(&mut self, sensor_fusion_filter_is_initializing: bool) {
        self.sensor_fusion_filter_is_initializing = sensor_fusion_filter_is_initializing;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<VehicleController>();
    }
    #[test]
    fn test_new() {
        let _vehicle_controller = VehicleController::new();
    }
}
