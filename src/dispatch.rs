use blackbox_logger::{GyroPidMessage, SetpointMessage};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::Watch;

//
// --- GYRO_PID ---
//

// The gyro_pid watch has three clients: the blackbox, the autopilot, and the OSD.
const GYRO_PID_WATCH_COUNT: usize = 3;
// Watch<Mutex, DataType, MaxReceivers>
static GYRO_PID_WATCH: Watch<CriticalSectionRawMutex, GyroPidMessage, GYRO_PID_WATCH_COUNT> = Watch::new();

// Type aliases make the function signatures much easier to read.
pub type GyroPidMessageSender =
    embassy_sync::watch::Sender<'static, CriticalSectionRawMutex, GyroPidMessage, GYRO_PID_WATCH_COUNT>;
pub fn gyro_pid_sender() -> GyroPidMessageSender {
    GYRO_PID_WATCH.sender()
}

pub type GyroPidReceiver =
    embassy_sync::watch::Receiver<'static, CriticalSectionRawMutex, GyroPidMessage, GYRO_PID_WATCH_COUNT>;
pub fn gyro_pid_receiver() -> GyroPidReceiver {
    GYRO_PID_WATCH.receiver().expect("gyro_pid receiver failed")
}

//
// --- SETPOINT ---
//

const SETPOINT_WATCH_COUNT: usize = 3;
static SETPOINT_WATCH: Watch<CriticalSectionRawMutex, SetpointMessage, SETPOINT_WATCH_COUNT> = Watch::new();

pub type SetpointMessageSender =
    embassy_sync::watch::Sender<'static, CriticalSectionRawMutex, SetpointMessage, SETPOINT_WATCH_COUNT>;
pub fn setpoint_sender() -> SetpointMessageSender {
    SETPOINT_WATCH.sender()
}

pub type SetpointReceiver =
    embassy_sync::watch::Receiver<'static, CriticalSectionRawMutex, SetpointMessage, SETPOINT_WATCH_COUNT>;
pub fn setpoint_receiver() -> SetpointReceiver {
    SETPOINT_WATCH.receiver().expect("setpoint receiver failed")
}
