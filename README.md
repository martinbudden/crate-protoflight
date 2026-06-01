# `protoflight` Rust Crate<br>![license](https://img.shields.io/badge/License-GPLv3_or_later-blue.svg) ![open source](https://badgen.net/badge/open/source/blue?icon=github)

## WORK IN PROGRESS

**THIS CRATE IS A WORK IN PROGRESS AND NOT YET READY FOR USE.**

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

## Protoflight name

I've called it Protoflight because:

1. It can be used to prototype new ideas.
2. It is related to "protean", meaning "able to change frequently or easily" or "versatile".
3. One of the meanings of "proto" is "primitive".
4. It pays homage to [Protea](https://en.wikipedia.org/wiki/Protea), which was the codename for the [Psion Series 5](https://en.wikipedia.org/wiki/Psion_Series_5)

## Crates used to build Protoflight

1. [vqm](https://crates.io/crates/vqm) - vectors, quaternions, and matrices.
2. [signal-filters](https://crates.io/crates/signal-filters) - filters library, including biquad, moving average, low-pass, and notch filters.
3. [pidsk-controller](https://crates.io/crates/pidsk-controller) - PID controller with optional feed forward and setpoint kick.
4. [imu-sensors](https://crates.io/crates/imu-sensors) - drivers for a variety of IMU (gyroscope and accelerometer) sensors.
5. [sensor-fusion](https://crates.io/crates/sensor-fusion) - sensor fusion filters for Attitude and Heading Reference Systems (AHRS).
6. [motor-mixers](https://crates.io/crates/motor-mixers) - converts desired throttle and roll, pitch, yaw torques into motor commands.
   Supports PWM and bidirectional `Dshot` protocols. Also includes RPM filters and dynamic idle control.
7. [radio-controllers](https://crates.io/crates/radio-controllers) - Drivers for SBUS, IBUS, Crossfire/ExpressLRS receivers..
8. [blackbox-logger](https://crates.io/crates/blackbox-logger) - based [implementation](https://github.com/thenickdude/blackbox) by Nicholas Sherlock (aka thenickdude).
9. [stream-buf](https://crates.io/crates/stream-buf) - simple serializer/deserializer.

## Steps required to reach "First Flight"

Protoflight has not yet achieved first flight. To do this, at least the following are required:

1. Get `imu-sensors` working with actual hardware.
2. Get `motor-mixers` to drive actual hardware.
3. Get `radio-controllers` to receive from actual hardware.
4. Create prototype board and put it all together.

## Original implementation

I originally implemented this program in C++:
[Protoflight](https://github.com/martinbudden/Protoflight).
