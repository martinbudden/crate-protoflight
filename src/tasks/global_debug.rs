#![allow(unused)]
/// Sets a global debug value if the "debug" feature is enabled.
/// Does absolutely nothing if the feature is disabled.
macro_rules! debug_set {
    ($mode:expr, $index:expr, $value:expr) => {
        #[cfg(feature = "debug")]
        $crate::tasks::GLOBAL_DEBUG.set($mode, $index, $value);
    };
}

macro_rules! debug_set_mode {
    ($mode:expr) => {
        #[cfg(feature = "debug")]
        $crate::tasks::GLOBAL_DEBUG.set_mode($mode);
    };
}

use core::sync::atomic::{AtomicI16, AtomicU8, Ordering};

/// `GLOBAL_DEBUG` is a global static protected by using atomic values.
pub static GLOBAL_DEBUG: GlobalDebug = GlobalDebug::new();

/// A lock-free, atomic version debug structure.
/// This can be safely placed in a global `static` without a Mutex.
pub struct GlobalDebug {
    pub mode: AtomicU8,
    pub values: [AtomicI16; Self::COUNT],
}

impl GlobalDebug {
    pub const COUNT: usize = 8;
    pub const COUNT_U8: u8 = 8;

    /// Create a new, zero-initialized atomic instance.
    pub const fn new() -> Self {
        Self {
            mode: AtomicU8::new(0),
            // Atomic arrays must be initialized element by element in a const context
            values: [
                AtomicI16::new(0),
                AtomicI16::new(0),
                AtomicI16::new(0),
                AtomicI16::new(0),
                AtomicI16::new(0),
                AtomicI16::new(0),
                AtomicI16::new(0),
                AtomicI16::new(0),
            ],
        }
    }
}

impl GlobalDebug {
    /// Sets the debug mode.
    pub fn set_mode(&self, mode: DebugMode) {
        self.mode.store(mode as u8, Ordering::Relaxed);
    }

    pub fn set_mode_u8(&self, mode: u8) {
        self.mode.store(mode, Ordering::Relaxed);
    }

    pub fn mode(&self) -> u8 {
        self.mode.load(Ordering::Relaxed)
    }

    /// Set a value completely lock-free.
    /// Can be safely called from sync functions, interrupts, or async tasks.
    pub fn set(&self, mode: DebugMode, index: usize, value: i16) {
        // Ensure index safety and verify the mode matches
        if index < Self::COUNT && mode as u8 == self.mode.load(Ordering::Relaxed) {
            // Overwrites the value instantly. The last caller wins.
            self.values[index].store(value, Ordering::Relaxed);
        }
    }

    /// Set a f32 value completely lock-free.
    pub fn set_f32(&self, mode: DebugMode, index: usize, value: f32) {
        #[allow(clippy::cast_possible_truncation)]
        self.set(mode, index, value as i16);
    }

    /// Return value at given index.
    pub fn _value(&self, index: usize) -> i16 {
        if index < Self::COUNT { self.values[index].load(Ordering::Relaxed) } else { 0 }
    }

    /// Returns an array of all the values.
    /// Approximately 15 to 40 CPU cycles.
    pub fn values(&self) -> [i16; Self::COUNT] {
        core::array::from_fn(|ii| self.values[ii].load(Ordering::Relaxed))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
//#[repr(u8)]
#[allow(missing_docs)]
#[allow(unused)]
pub enum DebugMode {
    #[default]
    None,
    CycleTime,
    Battery,
    GyroFiltered,
    Accelerometer,
    PidLoop,
    RcInterpolation,
    AngleRate,
    EscSensor,
    Scheduler,
    Stack,
    EscSensorRpm,
    EscSensorTmp,
    Altitude,
    Fft,
    FftTime,
    FftFreq,
    RxFrskySpi,
    RxSfhssSpi,
    GyroRaw,
    MultiGyroRaw,
    MultiGyroDiff,
    Max7456Signal,
    Max7456SpiClock,
    Sbus,
    Fport,
    Rangefinder,
    RangefinderQuality,
    OpticalFlow,
    LidarTf,
    AdcInternal,
    RunawayTakeoff,
    Sdio,
    CurrentSensor,
    Usb,
    SmartAudio,
    Rth,
    ItermRelax,
    AcroTrainer,
    RcSmoothing,
    RxSignalLoss,
    RcSmoothingRate,
    AntiGravity,
    DynLpf,
    RxSpektrumSpi,
    DshotRpmTelemetry,
    RpmFilter,
    DMax,
    AcCorrection,
    AcError,
    MultiGyroScaled,
    DshotRpmErrors,
    CrsfLinkStatisticsUplink,
    CrsfLinkStatisticsPwr,
    CrsfLinkStatisticsDown,
    Baro,
    AutopilotAltitude,
    DynIdle,
    FeedforwardLimit,
    Feedforward,
    BlackboxOutput,
    GyroSample,
    RxTiming,
    DLpf,
    VtxTramp,
    Ghst,
    GhstMsp,
    SchedulerDeterminism,
    TimingAccuracy,
    RxExpresslrsSpi,
    RxExpresslrsPhaselock,
    RxStateTime,
    GpsRescueVelocity,
    GpsRescueHeading,
    GpsRescueTracking,
    GpsConnection,
    Attitude,
    VtxMsp,
    GpsDop,
    Failsafe,
    GyroCalibration,
    AngleMode,
    AngleTarget,
    CurrentAngle,
    DshotTelemetryCounts,
    RpmLimit,
    RcStats,
    MagCalibration,
    MagTaskRate,
    Ezlanding,
    Tpa,
    STerm,
    Spa,
    Task,
    Gimbal,
    WingSetpoint,
    AutopilotPosition,
    Chirp,
    FlashTestPrbs,
    MavlinkTelemetry,
    AutopilotPid,
    PositionNav,
    #[allow(clippy::upper_case_acronyms)]
    COUNT,
}

impl_try_from_u8!(DebugMode);

#[allow(unused)]
impl DebugMode {
    /// Forgiving conversion from u8 to `DebugMode`, converts invalid values to default.
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::CycleTime,
            2 => Self::Battery,
            3 => Self::GyroFiltered,
            4 => Self::Accelerometer,
            5 => Self::PidLoop,
            6 => Self::RcInterpolation,
            7 => Self::AngleRate,
            8 => Self::EscSensor,
            9 => Self::Scheduler,
            10 => Self::Stack,
            11 => Self::EscSensorRpm,
            12 => Self::EscSensorTmp,
            13 => Self::Altitude,
            14 => Self::Fft,
            15 => Self::FftTime,
            16 => Self::FftFreq,
            17 => Self::RxFrskySpi,
            18 => Self::RxSfhssSpi,
            19 => Self::GyroRaw,
            20 => Self::MultiGyroRaw,
            21 => Self::MultiGyroDiff,
            22 => Self::Max7456Signal,
            23 => Self::Max7456SpiClock,
            24 => Self::Sbus,
            25 => Self::Fport,
            26 => Self::Rangefinder,
            27 => Self::RangefinderQuality,
            28 => Self::OpticalFlow,
            29 => Self::LidarTf,
            30 => Self::AdcInternal,
            31 => Self::RunawayTakeoff,
            32 => Self::Sdio,
            33 => Self::CurrentSensor,
            34 => Self::Usb,
            35 => Self::SmartAudio,
            36 => Self::Rth,
            37 => Self::ItermRelax,
            38 => Self::AcroTrainer,
            39 => Self::RcSmoothing,
            40 => Self::RxSignalLoss,
            41 => Self::RcSmoothingRate,
            42 => Self::AntiGravity,
            43 => Self::DynLpf,
            44 => Self::RxSpektrumSpi,
            45 => Self::DshotRpmTelemetry,
            46 => Self::RpmFilter,
            47 => Self::DMax,
            48 => Self::AcCorrection,
            49 => Self::AcError,
            50 => Self::MultiGyroScaled,
            51 => Self::DshotRpmErrors,
            52 => Self::CrsfLinkStatisticsUplink,
            53 => Self::CrsfLinkStatisticsPwr,
            54 => Self::CrsfLinkStatisticsDown,
            55 => Self::Baro,
            56 => Self::AutopilotAltitude,
            57 => Self::DynIdle,
            58 => Self::FeedforwardLimit,
            59 => Self::Feedforward,
            60 => Self::BlackboxOutput,
            61 => Self::GyroSample,
            62 => Self::RxTiming,
            63 => Self::DLpf,
            64 => Self::VtxTramp,
            65 => Self::Ghst,
            66 => Self::GhstMsp,
            67 => Self::SchedulerDeterminism,
            68 => Self::TimingAccuracy,
            69 => Self::RxExpresslrsSpi,
            70 => Self::RxExpresslrsPhaselock,
            71 => Self::RxStateTime,
            72 => Self::GpsRescueVelocity,
            73 => Self::GpsRescueHeading,
            74 => Self::GpsRescueTracking,
            75 => Self::GpsConnection,
            76 => Self::Attitude,
            77 => Self::VtxMsp,
            78 => Self::GpsDop,
            79 => Self::Failsafe,
            80 => Self::GyroCalibration,
            81 => Self::AngleMode,
            82 => Self::AngleTarget,
            83 => Self::CurrentAngle,
            84 => Self::DshotTelemetryCounts,
            85 => Self::RpmLimit,
            86 => Self::RcStats,
            87 => Self::MagCalibration,
            88 => Self::MagTaskRate,
            89 => Self::Ezlanding,
            90 => Self::Tpa,
            91 => Self::STerm,
            92 => Self::Spa,
            93 => Self::Task,
            94 => Self::Gimbal,
            95 => Self::WingSetpoint,
            96 => Self::AutopilotPosition,
            97 => Self::Chirp,
            98 => Self::FlashTestPrbs,
            99 => Self::MavlinkTelemetry,
            100 => Self::AutopilotPid,
            101 => Self::PositionNav,
            _ => Self::default(),
        }
    }
}
