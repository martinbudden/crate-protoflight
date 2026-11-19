use embassy_time::Instant;
use vqm::Vector3f32;

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "host", derive(derive_more::Display))]
#[cfg_attr(feature = "host", display("Mag{{x:reading.x,y:reading.y,z:reading.z}}"))]
pub struct MagnetometerMessage {
    pub timestamp: Instant,
    pub reading: Vector3f32,
}

impl Default for MagnetometerMessage {
    fn default() -> Self {
        Self::new(Instant::MIN, Vector3f32 { x: 0.0, y: 0.0, z: 0.0 })
    }
}

impl MagnetometerMessage {
    pub const fn new(timestamp: Instant, reading: Vector3f32) -> Self {
        Self { timestamp, reading }
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<MagnetometerMessage>();
    }
}
