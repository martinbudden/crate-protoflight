#![doc = include_str!("README.md")]

mod arming;
mod configuration;
mod features;
mod flight_controller;
mod flight_controller_config;
mod imu_filters;
mod rc_adjustments;
mod rx_message;
mod vehicle_control;
mod vehicle_controller;

#[allow(unused)]
pub use arming::{ArmingConfig, ArmingFlags};
pub use features::FeatureFlags;
pub use flight_controller::FlightController;
pub use flight_controller_config::{
    AntiGravityConfig, CrashFlipConfig, CrashRecoveryConfig, DMaxConfig, FlightControllerFiltersConfig,
    FlightModeConfig, GyroConfig, PidConfig, TpaConfig, YawSpinRecoveryConfig,
};
pub use imu_filters::{FilterAccGyro, ImuFilterBank, ImuFilterBankConfig};
pub use rc_adjustments::RcAdjustments;
pub use rx_message::{RcControls, RxMessage};
pub use vehicle_control::VehicleControl;
