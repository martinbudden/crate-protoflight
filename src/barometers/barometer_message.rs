use embassy_time::Instant;

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "host", derive(derive_more::Display))]
#[cfg_attr(feature = "host", display("Baro{{a:{altitude_m}, p:{pressure_pascals}}}"))]
pub struct BarometerMessage {
    pub timestamp: Instant,
    pub altitude_m: f32,
    pub altitude_m_i32: i32,
    pub pressure_pascals: f32,
}

impl Default for BarometerMessage {
    fn default() -> Self {
        Self::new(Instant::MIN, Self::STANDARD_PRESSURE_AT_SEA_LEVEL, Self::STANDARD_PRESSURE_AT_SEA_LEVEL)
    }
}

impl BarometerMessage {
    pub const STANDARD_PRESSURE_AT_SEA_LEVEL: f32 = 101_325.0;

    pub fn new(timestamp: Instant, pressure_pascals: f32, reference_pressure: f32) -> Self {
        let altitude_m = Self::calculate_altitude_meters(pressure_pascals, reference_pressure);
        Self {
            timestamp,
            altitude_m,
            #[allow(clippy::cast_possible_truncation)]
            altitude_m_i32: altitude_m as i32, // used by Blackbox
            pressure_pascals,
        }
    }
}

impl BarometerMessage {
    #[allow(unused)]
    pub fn calculate_altitude_meters(pressure: f32, pressure_at_reference_altitude: f32) -> f32 {
        44330.0 * (1.0 - (pressure / pressure_at_reference_altitude).powf(0.1903_f32))
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<BarometerMessage>();
    }
}
