# Global configuration

The `GlobalConfig` `struct` is a composition of all the individual config `struct`s in Protoflight and represents the total configuration of the system.

It is stored in the global static `GLOBAL_CONFIG` which is protected by the synchronization primitive `CriticalSectionRawMutex`.

Change to `GLOBAL_CONFIG` are signaled via two publish and subscribe channels: `CONFIG_PUB_SUB_CHANNEL` and `GYRO_PID_PUB_SUB_CHANNEL`.

`GYRO_PID_PUB_SUB_CHANNEL` is used to signal changes that are relevant to the `gyro_pid` task, namely changes to the PID gains and the IMU filters.

Other changes to `GLOBAL_CONFIG` are signalled using the `CONFIG_PUB_SUB_CHANNEL`.

This split is made so that the `gyro_pid` task (which runs at up to 8kHz) isn't slowed down by receiving irrelevant messages.

Changed configuration is published to these `PubSubChannels` using the data-carrying `enum` `GyroPidItem` for the gyro/pid pub/sub channel
and `ConfigItem` for the general pub/sub channel.
