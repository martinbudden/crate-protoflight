#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TaskInitError {
    BoardInitError,
    GyroPidSpawnFailed,
    MotorMixerSpawnFailed,
    RxSpawnFailed,

    TooManyGyroPidReceivers,
    TooManyRxMessageReceivers,
    TooManySetpointReceivers,
    TooManyAutopilotReceivers,

    ConfigPublisherFailed,
    ConfigSubscriberFailed,

    FastConfigPublisherFailed,
    FastConfigSubscriberFailed,

    BatteryPublisherFailed,
    BatterySubscriberFailed,

    BarometerPublisherFailed,
    BarometerSubscriberFailed,

    FailsafePublisherFailed,
    FailsafeSubscriberFailed,

    GpsPublisherFailed,
    GpsSubscriberFailed,

    MagnetometerPublisherFailed,
    MagnetometerSubscriberFailed,

    OpticalFlowPublisherFailed,
    OpticalFlowSubscriberFailed,

    RangefinderPublisherFailed,
    RangefinderSubscriberFailed,
}

impl TaskInitError {
    #[allow(unused)]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BoardInitError => "BoardInitError",
            Self::GyroPidSpawnFailed => "GyroPidSpawnFailed",
            Self::MotorMixerSpawnFailed => "MotorMixerSpawnFailed",
            Self::RxSpawnFailed => "RxSpawnFailed",
            Self::TooManyGyroPidReceivers => "TooManyGyroPidReceivers",
            Self::TooManyRxMessageReceivers => "TooManyRxMessageReceivers",
            Self::TooManySetpointReceivers => "TooManySetpointReceivers",
            Self::TooManyAutopilotReceivers => "TooManyAutopilotReceivers",

            Self::ConfigPublisherFailed => "ConfigPublisherFailed",
            Self::ConfigSubscriberFailed => "ConfigSubscriberFailed",

            Self::FastConfigPublisherFailed => "FastConfigPublisherFailed",
            Self::FastConfigSubscriberFailed => "FastConfigSubscriberFailed",

            Self::BatteryPublisherFailed => "BatteryPublisherFailed",
            Self::BatterySubscriberFailed => "BatterySubscriberFailed",

            Self::BarometerPublisherFailed => "BarometerPublisherFailed",
            Self::BarometerSubscriberFailed => "BarometerSubscriberFailed",

            Self::FailsafePublisherFailed => "FailsafePublisherFailed",
            Self::FailsafeSubscriberFailed => "FailsafeSubscriberFailed",

            Self::GpsPublisherFailed => "GpsPublisherFailed",
            Self::GpsSubscriberFailed => "GpsSubscriberFailed",

            Self::MagnetometerPublisherFailed => "MagnetometerPublisherFailed",
            Self::MagnetometerSubscriberFailed => "MagnetometerSubscriberFailed",

            Self::OpticalFlowPublisherFailed => "OpticalFlowPublisherFailed",
            Self::OpticalFlowSubscriberFailed => "OpticalFlowSubscriberFailed",

            Self::RangefinderPublisherFailed => "RangefinderPublisherFailed",
            Self::RangefinderSubscriberFailed => "RangefinderSubscriberFailed",
        }
    }
}
