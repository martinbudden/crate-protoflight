# Tasks

Protoflight has three realtime tasks (`pid_gyro`, `motor_mixer`, and `flight_control`) and a varying number of other tasks,
depending on the configuration.

Protoflight uses the [embassy](https://embassy.dev/book/) framework for task scheduling, task synchronization and inter-task
messaging.

Embassy has cooperative round-robing scheduling within an executor, and pre-emptive scheduling between executors.

Protoflight uses two embassy executors, and three distinct "execution contexts.":

1. Executor A (Core 1 - The "Hot" Core): A dedicated executor running only `gyro_pid`.
2. Executor B (Core 0 - The "Main" Core): The standard Embassy executor.
3. Interrupt Context (Core 0): This is for the high-speed logic for the `motor_mixer` and `radio_control` tasks.

| Core   | Context              | Tasks                                                          | Priority   |
| ------ | -------------------- | -------------------------------------------------------------- | ---------- |
| Core 1 | Dedicated Executor   | `gyro_pid` (8kHz)                                              | Real-time  |
| Core 0 | Interrupt Executor   | `motor_mixer` (8kHz*),<br> `flight_control` (~50Hz to 500Hz)   | High       |
| Core 0 | Main Thread Executor | Other tasks (< 100Hz )                                         | Lowest     |

Each task has an data context defined by a `struct`, so the `gyro_pid` task has `struct GyroPidContext` and so on.
Data contexts are private to each task.

Note: The `motor_mixer` task runs in sync with the `gyro_pid` task. On each loop of the task it updates the RPM notch filters parameters,
but it only outputs to the motors at typically 1kHz.

Tasks communicate with each other through Embassy's intercommunication data areas (IDAs) `signal`, `channel`, `watch`, and `pubsub`.
IDAs are guarded by synchronization primitives (eg mutexes), so that only one task can access an IDA at a time.
The synchronization primitive `CriticalSectionRawMutex` is used so that tasks can execute on different CPU cores without interfering with each other.

The `embassy-sync` crate provides the following structures to facilitate asynchronous communication and data sharing between tasks.

| Primitive       | Multi-receiver | Consume on read    | Behavior                                                                                   |
| --------------- | -------------- | ------------------ | ------------------------------------------------------------------------------------------ |
| `Signal`        | No             | Yes                | Single-slot notification optimized for exactly one consumer.                               |
| `Channel`       | Yes (MPMC)     | Yes                | A queue where each message is received by exactly one consumer.                            |
| `Watch`         | Yes            | No                 | Single-slot notification broadcast to multiple receivers.                                  |
| `PubSubChannel` | Yes            | No (waits for all) | Broadcasts a stream of events; producers wait until all active receivers copy the message. |

Protoflight uses them in the following way:

| IDA                         | Type   | Message              | Max Subscribers | Sending task(s)     | Receiving Task(s)        |
| --------------------------- | ------ | -------------------- | --------------- | ------------------- | ------------------------ |
| MOTOR_MIXER_SIGNAL          | Signal | MotorMixerMessage    | 1               | gyro_pid            | motor_mixer              |
| GPS_YAW_HEADING_SIGNAL      | Signal | GpsYawHeadingMessage | 1               | gps                 | gyro_pid                 |
| FLIGHT_CONTROL_WATCH        | Watch  | FlightControlMessage | 2               | flight_control      | gyro_pid, autopilot      |
| GYRO_PID_WATCH              | Watch  | GyroPidMessage       | 3               | gyro_pid            | blackbox, osd, autopilot |
| SETPOINT_WATCH              | Watch  | SetpointMessage      | 3               | gyro_pid            | blackbox, osd. autopilot |
| AUTOPILOT_WATCH             | Watch  | FlightControlMessage | 1               | autopilot           | flight_control           |
| FAST_CONFIG_PUB_SUB_CHANNEL | PubSub | FastConfigItem       | 1               | msp, flight_control | gyro_pid                 |
| CONFIG_PUB_SUB_CHANNEL      | PubSub | ConfigItem           | 8               | msp, flight_control | all except gyro_pid      |
| BAROMETER_PUB_SUB_CHANNEL   | PubSub | BarometerMessage     | 4               | barometer           | autopilot, msp, osd      |
| GPS_PUB_SUB_CHANNEL         | PubSub | GpsMessage           | 4               | gps                 | autopilot, msp, osd      |
| RANGEFINDER_PUB_SUB_CHANNEL | PubSub | RangefinderMessage   | 4               | rangefinder         | autopilot, msp, osd      |
