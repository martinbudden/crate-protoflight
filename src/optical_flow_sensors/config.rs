use crate::optical_flow_sensors::OpticalFlowType;

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OpticalFlowConfig {
    pub hardware: OpticalFlowType,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for OpticalFlowConfig {}

impl Default for OpticalFlowConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused)]
impl OpticalFlowConfig {
    pub const fn new() -> Self {
        Self { hardware: OpticalFlowType::None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<OpticalFlowConfig>();
        #[cfg(feature = "serde")]
        is_config::<OpticalFlowConfig>();
    }
}
