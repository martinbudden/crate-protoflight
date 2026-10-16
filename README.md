# `protoflight` Rust Crate<br>![License: GPLv3](https://img.shields.io/badge/License-GPLv3_or_later-blue.svg) ![open source](https://badgen.net/badge/open/source/blue?icon=github)

Protoflight is flight control software.

It has the following design goals (in no particular order).

1. A modular design that is built up from components in separate crates (see below).
2. Produce crates that are usable in their own right.
3. Be peformant. Support 8kHz Gyro/PID loop.
4. Support dual-core processors, in particular allow the Gyro/PID loop to have an entire core to itself.
5. Be (relatively) easy to learn and modify.
6. Give users the ability implement their own code or modifications.
7. Modular architecture to make it easier to identify which bit of code to modify, without impacting other code.
8. Be useful to people who want to experiment with and customize a flight controller.
9. Be useful to someone who wants to understand how flight control software works.
10. Be Betaflight "Tool compatible". This is be able to use the Betaflight Configurator and the Betaflight Blackbox Explorer.

> **⚠️ Note:** This crate is currently under active development.
>
> It is suitable for experimental use and development, but not for active deployment.

## Collaboration crates

The following crates were written in conjunction with Protoflight:

1. [vqm](https://crates.io/crates/vqm) - vectors for acc and gyro readings, quaternions for sensor fusion and orientation, and matrices for Kalman filters.
2. [signal-filters](https://crates.io/crates/signal-filters) - for filtering acc and gyro values, including RPM filtering.
3. [pidsk-controller](https://crates.io/crates/pidsk-controller) - PID controllers used for roll, pitch and yaw control,
   altitude and position hold, and path following.
4. [imu-sensors](https://crates.io/crates/imu-sensors) - drivers to obtain acc and gyro readings.
5. [sensor-fusion](https://crates.io/crates/sensor-fusion) - Mahony and Madgwick filters for sensor fusion of acc and gyro values.
   Kalman filters for altitude and position estimation.
6. [motor-mixers](https://crates.io/crates/motor-mixers) - converts desired throttle and roll, pitch, yaw torques into motor commands.
   Supports PWM and bidirectional `Dshot` protocols. Also includes RPM filters and dynamic idle control.
7. [radio-controllers](https://crates.io/crates/radio-controllers) - Drivers for SBUS, IBUS, Crossfire/ExpressLRS receivers.
   Mode activation conditions. Setting radio control rates.
8. [blackbox-logger](https://crates.io/crates/blackbox-logger) - encodes flight data into Blackbox format.
9. [stream-buf](https://crates.io/crates/stream-buf) - simple serializer/deserializer used by MSP.

The general aim is to move functionality (once it is stable) out of Protoflight into its own crate. So for example,
I expect to, at some point, move the barometer, magnetometer, gps etc functionality into their own crates.

I'm also considering moving the Board Support Packages into a separate crate.

## Overview

Here's a simplified overview of the system.

```text
                                                     ┌───────┐
                                                     │ RADIO │
                                                     └─┬───┬─┘
                                                       │   │
                                            RcCommands ▼   ▼ RcModes
                                                       │   │
 ┌─────┐  ┌────────┐              Acc              ┌───┴───┴────┐
 │     │  │ SIGNAL ├───────────────▶───────────────┤ FLIGHT     │   ┌───────┐
 │ IMU ├─▶┤ FILTER │ Acc  ┌──────────┐             │ CONTROLLER ├─▶─┤ MOTOR │
 │     │  │        ├──▶───┤ MADGWICK ├─────▶───────┤ (PIDS)     │   │ MIXER │
 └──┬──┘  └────────┘ Gyro │ FILTER   │ Orientation └─────┬──────┘   └───┬───┘
    │                     └────┬─────┘                   │              │
    ▼ Acc(unfiltered)          │                         │      Torques ▼
    │                          ▼ Orientation             │              │
┌───┴────────┐                 │                         │          ┌───┴────┐
│    IMU     ├────◀────────────┘                Commands ▲          │ MOTORS │
│ INTEGRATOR |                                           │          │ SERVOS │
└───┬────────┘                                           │          └────────┘
    │                                                    │
  delta_pos                                              │
  delta_vel         ┌────────┐        Position     ┌─────┴─────┐
    └───────────▶───┤ KALMAN ├───────────▶─────────┤ AUTOPILOT │
                    │ FILTER │        Velocity     └───────────┘
                    └────┬───┘
                         │
  GPS, Barometer, etc ─▶─┘

      ┌────────┐      ┌────────┐
      │ GLOBAL │      │ GLOBAL │
      │ CONFIG |      │ DEBUG  |
      └────────┘      └────────┘
```

GLOBAL CONFIG is protected by a mutex and is accessible by all tasks.

GLOBAL DEBUG is protected by atomics and is accessible by all tasks.

`acc`, `gyro`, `delta_pos`, `delta_vel`, `position`, and `velocity` are all 3D vectors.

`orientation` is a quaternion.

`RcCommand`s are the stick inputs of roll, pitch, yaw, and throttle.
`RcMode`s are set using the radio's auxiliary channels.

Some subtleties not shown on the diagram:

- Failsafe is not shown. When there is no signal from the radio, failsafe interacts with the Flight Controller and/or the Autopilot
  to land the aircraft or bring it home.
- `RcMode`s are shown as inputs to the Flight Controller. This is their main purpose, but they are used as inputs to many parts of the system.
- The Autopilot receives `RcCommand`s and `RcMode`s from the Radio.
- If using Dshot, the signal filter dynamically adjusts the filters according to motor RPM.
- If a magnetometer is present it is used by both the Kalman filter, and the Madgwick filter (to correct the yaw component of orientation).
- When the aircraft is in motion, the Kalman filter's velocity vector is used to correct the yaw component of the Madgwick filter's orientation.

Integrating the acc and gyro values to produce `delta_pos` and `delta_vel` vectors means we can use a Linear Kalman Filter (LKF)
with 9x9 state matrices. (Passing acc and gyro values to the Kalman filter directly would require an Extended Kalman Filter (EKF)
with 15x15 state matrices - this is **much** more computationally expensive.)

## Board Support Packages

Board Support Packages (BSPs) are in the `boards` module (`src/boards`).

For each target board (eg OpenFC-Lite, Speedybee v4 etc) the BSP defines what pins are used by each peripheral,
what type of IMU is used etc.

PRs for new BSPs, and corrections and enhancements to existing BSPs are most welcome.

### Adding a new board

To add support for a new board you need to:

1. Create a rust module for that board in `src/boards/targets`, eg `src/boards/targets/openfc_lite.rs`.
2. Create a feature flag for the board in `Cargo.toml`, eg `openfc_lite`.
3. Define the the microcontroller that board uses and the features that board supports, eg:

```text
openfc_lite = [
    "rp235xb",
    # "multicore",
    "sdcard",
    "_common",
]
```

## Steps required to reach "First Flight"

Protoflight has not yet achieved first flight. To do this, at least the following are required:

1. Further testing of `imu-sensors` on actual hardware.
2. Further testing of `motor-mixers` on actual hardware.
3. Further testing of  `radio-controllers` on actual hardware.

## Protoflight name

I've called it Protoflight because:

1. It can be used to prototype new ideas.
2. It is related to "protean", meaning "able to change frequently or easily" or "versatile".
3. One of the meanings of "proto" is "primitive".
4. It pays homage to [Protea](https://en.wikipedia.org/wiki/Protea), which was the codename for the [Psion Series 5](https://en.wikipedia.org/wiki/Psion_Series_5)

## Original implementation

I originally implemented this program in C++:
[Protoflight](https://github.com/martinbudden/Protoflight).
