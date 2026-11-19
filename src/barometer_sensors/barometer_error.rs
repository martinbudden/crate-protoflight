use crate::i2c_bus::I2cError;

#[allow(unused)]
#[derive(Debug)]
pub enum BarometerError<E> {
    I2c(E),
    InvalidChipId { expected: u8, actual: u8 },
    InvalidCalibration,
    InvalidData,
    TemperatureOversamplingNotSet,
    PressureOversamplingNotSet,
}

pub type BarometerI2cError = BarometerError<I2cError>;

#[cfg(test)]
mod test_traits {
    use super::*;
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    fn normal_types() {
        is_normal::<BarometerI2cError>();
    }
}
