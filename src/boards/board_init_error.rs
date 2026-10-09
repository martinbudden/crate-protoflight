#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoardInitError {
    ImuNotAvailable,
    ImuError,
    GyroInterruptNotAvailable,
    GyroInterruptError,

    Spi0InitFailed,
    Spi1InitFailed,
    Spi2InitFailed,
    Spi3InitFailed,
    Spi4InitFailed,
    Spi5InitFailed,

    SdCardNotAvailable,
    SdCardError,

    Max7456NotAvailable,
    Max7456Error,

    SerialRxUartNotAvailable,
    SerialRxUartError,
    MspUartNotAvailable,
    MspUartError,
    EscSensorUartNotAvailable,
    EscSensorUartError,
    UartNotAvailable,
    UartError,

    SensorsI2cNotAvailable,
    SensorsI2cError,

    MotorDriverNotAvailable,
    MotorDriverError,
    MotorProtocolNotSupported,
    LedcTimerConfigFailed,
    LedcChannel0ConfigFailed,
    LedcChannel1ConfigFailed,
    LedcChannel2ConfigFailed,
    LedcChannel3ConfigFailed,
}
