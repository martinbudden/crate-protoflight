# Boards

Boards readme placeholder.

```text
                ┌──────────────────────┐
                │       Board<I>       │
                │                      │
                │  imu: I              │
                │  motors: MotorDriver │
                │  SPI devices...      │
                │  UART devices...     │
                └──────────┬───────────┘
                        │
                        │ move
                        ▼
                ┌──────────────────────┐
                │    ImuContext<I>     │
                │                      │
                │  imu: I              │
                └──────────┬───────────┘
                        │
                        ▼
                    ┌───────────┐
                    │ imu_task  │
                    └─────┬─────┘
                        │
                    ImuDevice trait
                        │
            ┌──────────────┼──────────────┐
            ▼              ▼              ▼
    Lsm6ds<I2c>    Imu426xx<Spi>    ImuMock
```
