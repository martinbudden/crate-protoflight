# Tasks

Protoflight has three realtime tasks (`pid_gyro`, `motor_mixer`, and `radio_control`) and a varying number of other tasks,
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
| Core 0 | Interrupt Executor   | `motor_mixer` (8kHz*),<br> `radio_controller` (~50Hz to 500Hz) | High       |
| Core 0 | Main Thread Executor | Other tasks (< 100Hz )                                         | Lowest     |

Each task has an data context defined by a `struct`, so the `gyro_pid` task has `struct GyroPidContext` and so on.
Data contexts are private to each task.

Note: The `motor_mixer` task runs in sync with the `gyro_pid` task. On each loop of the task it updates the RPM notch filters parameters,
but it only outputs to the motors at typically 1kHz.

Tasks communicate with each other through Embassy's intercommunication data areas (IDAs) `signal`, `watch`, `channel`, and `pubsub`.
IDAs are guarded by synchronization primitives (eg mutexes), so that only one task can access an IDA at a time.
The synchronization primitive `CriticalSectionRawMutex` is used so that tasks can execute on different CPU cores without interfering with each other.

The `embassy-sync` crate provides the following structures to facilitate asynchronous communication and data sharing between tasks.

1. `Signal`: A single-slot primitive for sending the latest value to exactly one consumer.

2. `Watch`: A single-slot primitive that allows multiple receivers to concurrently await the latest value.

3. `Channel`: A queue (MPMC) for sending values where each message is received by exactly one consumer from a pool.

4. `PubSubChannel`: every message is received by all active consumers.

Protoflight uses them in the following way:

| IDA                              | Type   | Sending task(s) | Receiving Task(s)     |
| -------------------------------- | ------ | --------------- | --------------------- |
| MOTOR_MIXER_SIGNAL               | Signal | gyro_pid        | motor_mixer           |
| RADIO_WATCH                      | Watch  | radio           | gyro_pid              |
| GYRO_PID_WATCH                   | Watch  | gyro_pid        | blackbox, osd         |
| SETPOINT_WATCH                   | Watch  | gyro_pid        | blackbox, osd         |
| FAST_CONFIG_PUB_SUB_CHANNEL      | PubSub | msp, radio      | gyro_pid              |
| CONFIG_PUB_SUB_CHANNEL           | PubSub | msp, radio      | all except gyro_pid   |
| FAST_SENSOR_DATA_PUB_SUB_CHANNEL | PubSub | gps             | gyro_pid              |
| SENSOR_DATA_PUB_SUB_CHANNEL      | PubSub | barometer, gps  | all except gyro_pid   |
