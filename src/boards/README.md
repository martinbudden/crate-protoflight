# Boards

The boards directory contains the Board Support Packages (BSP) for the boards supported by Protoflight.

For example the Speedy Bee F404 V4 board is initialized in the `speedybee_f405_v4.rs` BSP.

It is there that the pins are assigned, the SPI and I2C devices, the UARTs etc created,
and they are assigned to the Protoflight objects.

There is no visibility to the Protoflight app of this low level hardware, all Protoflight
sees is the higher level objects, ie `Imu`, `MotorDriver` etc.

The low level hardware is abstracted away in the BSP, partly by Protoflight and partly by the `embassy` framework.
