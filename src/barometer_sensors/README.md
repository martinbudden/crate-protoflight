# Barometer Sensors

Placeholder.

```text
            BarometerError<E>
                ▲
                │
        ┌────────┴────────┐
        │                 │
    DPS310             BMP085
        │                 │
        └────────┬────────┘
                │
            BarometerI2c
                │
        SharedI2cBus
                │
        I2cDeviceBlocking
```

```text
Barometer task
      │
      │ await
      ▼
 SharedI2cExt
      │
      │ await mutex
      ▼
 Embassy Mutex
      │
      │ owns bus exclusively
      ▼
I2cDeviceBlocking
      │
      │ blocking_write/read
      ▼
  STM32 I2C
```

The important benefit is that contention for the shared bus is asynchronous, while the actual hardware transaction remains blocking.
That fits your decision to sacrifice the I²C DMA streams while keeping the UART/SPI DMA resources available for the higher-priority work.

```text
                         BackgroundExecutor
                                │
                         ┌──────┴──────┐
                         │             │
                    Barometer       Magnetometer
                         │             │
                         └──────┬──────┘
                                │
                              await
                                │
                         SharedI2cExt
                                │
                         lock().await
                                │
                         Embassy Mutex
                                │
                       ┌────────┴────────┐
                       │  exclusive bus │
                       └────────┬────────┘
                                │
                         I2cDeviceBlocking
                                │
                     blocking_write/read
                                │
                            STM32 I²C
```

`bus.blocking_write_read(...)` runs synchronously.
That's acceptable because the transaction is relatively short and, importantly, these I²C users are background tasks rather than your realtime tasks.
