# Global Sensor Data

The `SensorData` `struct` is a composition of all the individual sensor data `struct`s in Protoflight.

It is stored in the global static `SENSOR_DATA` which is protected by the synchronization primitive `CriticalSectionRawMutex`.

Changes to `SENSOR_DATA` are signaled via the publish and subscribe channels: `SENSOR_DATA_PUB_SUB_CHANNEL`.

New sensor data is published to this `PubSubChannel` using the data-carrying `enum` `SensorDataItem`.
