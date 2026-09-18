# Boards

The boards directory contains the Board Support Packages (BSP) for the boards supported by Protoflight.

For example the Speedy Bee F404 V4 board is initialized in the `speedybee_f405_v4.rs` BSP.

It is there that the pins are assigned, the SPI and I2C devices, the UARTs etc created,
and they are assigned to the Protoflight objects.

There is no visibility to the Protoflight app of this low level hardware, all Protoflight
sees is the higher level objects, ie `Imu`, `MotorDriver` etc.

The low level hardware is abstracted away in the BSP, partly by Protoflight and partly by the `embassy` framework.

## STM32 hardware allocation

| Hardware | Assignment                               |
| -------- | ---------------------------------------- |
| VCP      | MSP configuration                        |
| SPI1     | IMU (Acc/Gyro)                           |
| SPI2     | Blackbox                                 |
| SPI3     | Analog OSD (MAX7456)                     |
| I2C1     | Barometer / Magnetometer                 |
| UART1    | Digital VTX / OSD                        |
| UART2    | Serial Radio Receiver (ELRS / Crossfire) |
| UART3    | GPS Module                               |
| UART4/6  | ESC Telemetry                            |

## Raspberry Pi Pico hardware allocation

```text
            ┌────────────────────┐
            │ RP Microcontroller │
            └────────────────────┘
                    │              
        ┌───────────┴─────────────────┐─────────────────────────┐─────────────────────┐
        ▼                             ▼                         ▼                     ▼
 ┌───────────────┐              ┌─────────────┐           ┌─────────────┐       ┌─────────────┐
 │ Hardware Core │              │ PIO 0 Block │           │ PIO 1 Block │       │ PIO 2 Block │
 └───────────────┘              └─────────────┘           └─────────────┘       └─────────────┘
    │                              │                         │                     │
    ├── USB VCP (Configurator)     ├── SM 0 (GPS TX)         ├── SM 0 (Motor 1)    ├── SM 0 (Motor 5)
    ├── UART1 (Radio)              ├── SM 1 (GPS RX)         ├── SM 1 (Motor 2)    ├── SM 1 (Motor 6)
    ├── UART0 (VTX/Digital OSD)    ├── SM 2 (SPI Analog OSD) ├── SM 2 (Motor 3)    ├── SM 2 (Motor 7)
    ├── SPI0 (Gyro)                └── SM 3 (Spare)          └── SM 3 (Motor 4)    └── SM 3 (Motor 8)
    ├── SPI1 (Blackbox)
    └── I2C0 (Baro/Mag)
```
