#![doc = include_str!("../README.md")]
#![no_std]
#![deny(clippy::unwrap_used)]
//#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![warn(unused_results)]
#![warn(clippy::pedantic)]
#![warn(clippy::doc_paragraphs_missing_punctuation)]
#![allow(clippy::inline_always)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]

mod autopilot;
mod config;
mod dispatch;
mod display;
mod flight;
mod gps;
mod multiwii_serial_protocol;
mod osd;
mod sensors;
mod tasks;
mod vtx;

use crate::config::{CONFIG_PUB_SUB_CHANNEL, GLOBAL_CONFIG, GYRO_PID_PUB_SUB_CHANNEL};
use crate::dispatch::{gyro_pid_sender, setpoint_sender};
use crate::flight::{FlightController, ImuFilterBank, RcAdjustments};
use crate::multiwii_serial_protocol::Msp;
use crate::tasks::radio_task::{radio_receiver, radio_sender};
use crate::tasks::{GYRO_CTX, GyroPidContext, gyro_pid_task};
use crate::tasks::{MOTOR_MIXER_CTX, MotorMixerContext, motor_mixer_task};
use crate::tasks::{MSP_CTX, MspContext, msp_task};
use crate::tasks::{RADIO_CTX, RadioContext, radio_task};

#[cfg(feature = "blackbox")]
use crate::{
    dispatch::{gyro_pid_receiver, setpoint_receiver},
    tasks::{BLACKBOX_CTX, BlackboxContext, blackbox_task},
};
#[cfg(feature = "osd")]
use crate::{
    osd::Osd,
    tasks::{OSD_CTX, OsdContext, osd_task},
};
#[cfg(feature = "blackbox")]
use blackbox_logger::{Blackbox, FieldSelect, drivers::sd_card::MockSdCard};

use imu_sensors::ImuAxesOrder;
use imu_sensors::{ImuMock, MockImuBus};
//use sequential_storage::cache::{self, NoCache};
//use sequential_storage::map::SerializationError;

use motor_mixers::{MotorMixerCommon, MotorMixerQuadXPwm};
use sensor_fusion::MadgwickFilterf32;

use radio_controllers::{Rates, RcModes};

use embassy_executor::Spawner;
#[cfg(feature = "multicore")]
use embassy_rp::multicore::{Stack, spawn_core1};

// Core 1 needs its own stack space in RAM
#[cfg(feature = "multicore")]
static mut CORE1_STACK: Stack<4096> = Stack::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    env_logger::init();

    #[cfg(feature = "blackbox")]
    let mut config = GLOBAL_CONFIG.lock().await;
    #[cfg(not(feature = "blackbox"))]
    let config = GLOBAL_CONFIG.lock().await;

    let gyro_pid_ctx = GYRO_CTX.init(GyroPidContext {
        radio_receiver: radio_receiver(),
        gyro_pid_sender: gyro_pid_sender(),
        setpoint_sender: setpoint_sender(),
        gyro_pid_subscriber: GYRO_PID_PUB_SUB_CHANNEL.subscriber().expect("failed to create GYRO_PID subscriber"),
        imu: ImuMock::new(MockImuBus::new(), ImuAxesOrder::XPOS_YPOS_ZPOS),
        imu_filters: ImuFilterBank::with_config(config.imu_filter_bank),
        sensor_fusion: MadgwickFilterf32::new(),
        flight_controller: FlightController::new(),
    });

    let motor_mixer_ctx = MOTOR_MIXER_CTX.init(MotorMixerContext {
        motor_mixer: MotorMixerQuadXPwm::new(MotorMixerCommon::with_config(config.mixer, config.motor)),
    });

    let msp_ctx = MSP_CTX.init(MspContext {
        msp: Msp::new(),
        read_buf: [0u8; MspContext::READ_BUF_SIZE],
        write_buf: [0u8; MspContext::WRITE_BUF_SIZE],
    });

    let radio_ctx = RADIO_CTX.init(RadioContext {
        radio_sender: radio_sender(),
        config_subscriber: CONFIG_PUB_SUB_CHANNEL.subscriber().expect("failed to create RADIO config subscriber"),
        rates: Rates::new(config.rates),
        rc_modes: RcModes::new(),
        rc_adjustments: RcAdjustments::new(),
    });

    #[cfg(feature = "blackbox")]
    let blackbox_ctx = {
        config.blackbox.fields_disabled_mask = FieldSelect::PID_STERM_ROLL
        | FieldSelect::PID_STERM_PITCH
        | FieldSelect::PID_STERM_YAW
        | FieldSelect::PID_KTERM
        //| FieldSelect::PID
        | FieldSelect::RSSI
        | FieldSelect::SETPOINT
        //| FieldSelect::GYRO_UNFILTERED
        | FieldSelect::MOTOR_RPM
        | FieldSelect::BATTERY_VOLTAGE
        | FieldSelect::BATTERY_CURRENT
        | FieldSelect::BAROMETER
        | FieldSelect::RANGEFINDER
        | FieldSelect::ATTITUDE
        | FieldSelect::MAGNETOMETER;

        let mut blackbox = Blackbox::new(config.blackbox);
        blackbox.init();

        BLACKBOX_CTX.init(BlackboxContext {
            gyro_pid_receiver: gyro_pid_receiver(),
            setpoint_receiver: setpoint_receiver(),
            blackbox,
            buffer: [0u8; 1024],
            pos: 0,
            sd_card: MockSdCard::new("blackbox_log.bbl"),
        })
    };

    #[cfg(feature = "osd")]
    let osd_ctx = OSD_CTX.init(OsdContext {
        gyro_pid_receiver: gyro_pid_receiver(),
        setpoint_receiver: setpoint_receiver(),
        osd: Osd::new(),
    });

    drop(config); // unlocks

    /*
    TODO: gyro_pid on core1 and motor_mixer and radio on high priority interrupt driven spawner
    // Launch Core 1
    unsafe {
        spawn_core1(p.CORE1, &mut CORE1_STACK, core1_entry);
    }    // Create the task tokens and spawn the tasks.
    // 2. Start an InterruptExecutor on Core 0 for 1kHz tasks
    let high_spawner = EXECUTOR_HIGH.start(interrupt::SWI_IRQ_0);
    high_spawner.spawn(motor_mixer_task(motor_mixer_ctx).expect("Failed to create motor mixer task")); // No receiver needed, since it uses a SIGNAL
    high_spawner.spawn(radio_task(radio_ctx).expect("Failed to create radio task"));
    */

    // The gyro_pid task calculates the motor commands, sends them immediately to the motor_mixer task
    // and then updates the GyroPidMessage and sends it.
    spawner.spawn(gyro_pid_task(gyro_pid_ctx).expect("Failed to create gyro_pid task"));
    spawner.spawn(motor_mixer_task(motor_mixer_ctx).expect("Failed to create motor mixer task")); // No receiver needed, since it uses a SIGNAL
    spawner.spawn(radio_task(radio_ctx).expect("Failed to create radio task"));
    spawner.spawn(msp_task(msp_ctx).expect("Failed to create msp task"));

    // The blackbox and OSD tasks use a Watch.
    #[cfg(feature = "blackbox")]
    spawner.spawn(blackbox_task(blackbox_ctx).expect("Failed to create blackbox task"));
    #[cfg(feature = "osd")]
    spawner.spawn(osd_task(osd_ctx).expect("Failed to create OSD task"));
}
