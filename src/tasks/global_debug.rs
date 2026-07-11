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

    /// Return value at given index.
    pub fn value(&self, index: usize) -> i16 {
        if index < Self::COUNT { self.values[index].load(Ordering::Relaxed) } else { 0 }
    }

    /// Returns an array of all the values.
    pub fn values(&self) -> [i16; Self::COUNT] {
        core::array::from_fn(|ii| self.values[ii].load(Ordering::Relaxed))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
//#[repr(u8)]
#[allow(missing_docs)]
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
    Count,
}
