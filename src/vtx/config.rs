#![cfg(feature = "vtx")]

#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct VtxConfig {
    /// Sets freq in MHz if band=0.
    pub frequency_mhz: u16,
    /// sets out-of-range pit mode frequency.
    pub pit_mode_frequency_mhz: u16,
    /// Band: 1=A, 2=B, 3=E, 4=F(Airwaves/Fatshark), 5=Raceband.
    pub band: u8,
    /// Channel: 1-8.
    pub channel: u8,
    /// Power: 0 = lowest.
    pub power: u8,
    /// Min power while disarmed.
    pub low_power_disarm: u8,
    /// Prepend 0xff before sending frame.
    pub softserial_alt: u8,
}

impl Default for VtxConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for VtxConfig {}

impl VtxConfig {
    pub const fn new() -> Self {
        Self {
            frequency_mhz: 5740,
            pit_mode_frequency_mhz: 0,
            band: 4,
            channel: 1,
            power: 1,
            low_power_disarm: 1,
            softserial_alt: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_serde<T: Serialize + MaxSize + for<'a> Deserialize<'a>>() {}
    #[cfg(feature = "storage")]
    fn is_storage<T: for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<VtxConfig>();
        #[cfg(feature = "serde")]
        is_serde::<VtxConfig>();
        #[cfg(feature = "storage")]
        is_storage::<VtxConfig>();
    }
    #[test]
    fn test_new() {
        let config = VtxConfig::new();
        assert_eq!(5740, config.frequency_mhz);
    }
}
