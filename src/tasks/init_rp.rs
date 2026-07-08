#![cfg(feature = "rp2350")]
#![allow(unused)]

#[cfg(feature = "multicore")]
use embassy_rp::multicore::{Stack, spawn_core1};

use cyw43_pio::PioSpi;
use embassy_rp::{
    Peri, bind_interrupts, dma,
    gpio::{Level, Output},
    peripherals::FLASH,
    peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3, DMA_CH4, PIO0, SPI0, SPI1},
    pio,
    pio::InterruptHandler as PioInterruptHandler,
    spi::{Async, Config as SpiConfig, Spi},
};
use embassy_time::Delay; // Pulled from your cyw43-pio dependency

// Binds the global hardware DMA vectors.
// This creates the type validation struct "Irqs" required by Spi::new.
bind_interrupts!(pub struct Irqs {
    // Both SPI0 and SPI1 use these DMA channels to handle async wake ups
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>,
                 dma::InterruptHandler<DMA_CH1>,
                 dma::InterruptHandler<DMA_CH2>,
                 dma::InterruptHandler<DMA_CH3>;

    // Used by your 3rd PIO-backed SPI device
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

//use embedded_hal_async::spi::SpiDevice;
use embedded_hal_bus::spi::ExclusiveDevice;
//use imu_sensors::AccUnits::G;

// --- Device 1: Hardware SPI0 (Gyroscope) ---
// Tied to SPI0 running asynchronously via the DMA system
pub type GyroSpiDevice = ExclusiveDevice<Spi<'static, SPI0, Async>, Output<'static>, Delay>;

// --- Device 2: Hardware SPI1 (Blackbox SD Card) ---
// Tied to SPI1 running asynchronously via the DMA system
pub type BlackboxSpiDevice = ExclusiveDevice<Spi<'static, SPI1, Async>, Output<'static>, Delay>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlackboxInitError {
    FeatureDisabled,
}
// --- Device 3: PIO0 Backed SPI (Auxiliary Peripheral) ---
// Fully concrete representation using State Machine 0 on the PIO0 block
pub type AuxiliaryPioSpiDevice = ExclusiveDevice<PioSpi<'static, PIO0, 0>, Output<'static>, Delay>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuxiliaryPioInitError {
    /// The blackbox logging software module feature was disabled at build time.
    FeatureDisabled,
}

// --- 1. RASPBERRY PI RP2350 ARCHITECTURE CONFIGURATION ---
#[cfg(feature = "max7456")]
pub type ConcreteSpi = embassy_rp::spi::Spi<'static, embassy_rp::peripherals::SPI0, embassy_rp::spi::Async>;

#[cfg(feature = "max7456")]
pub type SharedDisplay = Mutex<CriticalSectionRawMutex, DisplayPortMax7456<&'static mut ConcreteSpi>>;

pub fn init_rp() -> (
    Result<GyroSpiDevice, core::convert::Infallible>,
    Result<BlackboxSpiDevice, BlackboxInitError>,
    Result<AuxiliaryPioSpiDevice, AuxiliaryPioInitError>,
    Peri<'static, FLASH>,
) {
    // Take ownership of the raw RP2350 hardware peripherals block
    let peripherals = embassy_rp::init(Default::default());

    let gyro_spi: Result<
        ExclusiveDevice<
            Spi<'_, embassy_rp::peripherals::SPI0, embassy_rp::spi::Async>,
            Output<'_>,
            embassy_time::Delay,
        >,
        core::convert::Infallible,
    > = {
        let mut spi_config = SpiConfig::default();
        spi_config.frequency = 10_000_000;

        // Notice: Irqs is completely omitted from the parameters here.
        let spi_bus = Spi::new(
            peripherals.SPI0,
            peripherals.PIN_18,  // CLK defined internally
            peripherals.PIN_19,  // MOSI defined internally
            peripherals.PIN_16,  // MISO defined internally
            peripherals.DMA_CH0, // TX DMA
            peripherals.DMA_CH1, // RX DMA
            Irqs,
            spi_config,
        );
        let cs_pin = Output::new(unsafe { core::ptr::read(&peripherals.PIN_17) }, Level::High);
        ExclusiveDevice::new(spi_bus, cs_pin, embassy_time::Delay)
    };

    #[cfg(feature = "blackbox")]
    let blackbox_spi = {
        let mut spi_config = SpiConfig::default();
        // When an SD card boots up, it starts in native SD mode.
        // To force it into SPI mode, the driver sends raw command sequences (CMD0, CMD8, ACMD41).
        // During this initial negotiation, cards only accept a clock speed between 100 kHz and 400 kHz.
        // Passing anything higher will cause the card to fail to answer.
        spi_config.frequency = 400_000;
        // TODO: increase SPI frequency to 20_000_000 after initialization.

        // Notice: Irqs is completely omitted from the parameters here.
        let spi_bus = Spi::new(
            peripherals.SPI1,
            peripherals.PIN_10,  // CLK defined internally
            peripherals.PIN_11,  // MOSI defined internally
            peripherals.PIN_12,  // MISO defined internally
            peripherals.DMA_CH2, // TX DMA
            peripherals.DMA_CH3, // RX DMA
            Irqs,
            spi_config,
        );
        let cs_pin = Output::new(unsafe { core::ptr::read(&peripherals.PIN_13) }, Level::High);
        // Map the infallible output into an Ok Result variant matching the outer structure
        ExclusiveDevice::new(spi_bus, cs_pin, embassy_time::Delay).map_err(|_| unreachable!())
    };

    #[cfg(not(feature = "blackbox"))]
    let blackbox_spi = Err(BlackboxInitError::FeatureDisabled);
    // --- Device 3: PIO0 Backed SPI (Auxiliary Peripheral) ---
    /*let aux_pio_spi = {
        // Safe global allocation layout for PIO. Keeps the references valid forever ('static)
        static PIO_CELL: static_cell::StaticCell<embassy_rp::pio::Pio<'static, PIO0>> = static_cell::StaticCell::new();
        let pio = PIO_CELL.init(embassy_rp::pio::Pio::new(peripherals.PIO0, Irqs));

        // Safely extract the exact structural elements requested by PioSpi::new
        let pio_irq   = unsafe { core::ptr::read(&pio.irq0) }; // Extract target Irq line
        let sm0_block = unsafe { core::ptr::read(&pio.sm0) };  // Extract state machine token

        // Initialize the concrete structural pin types required by the 8-arg signature
        let clk_pin  = pio.common.make_pio_pin(peripherals.PIN_2);
        let mosi_pin = Output::new(unsafe { core::ptr::read(&peripherals.PIN_3) }, Level::High);
        let miso_pin = pio.common.make_pio_pin(peripherals.PIN_4);

        // Independent CS pin line to prevent bus contentions or device cross-talk bugs
        let cs_pin = Output::new(unsafe { core::ptr::read(&peripherals.PIN_5) }, Level::High);

        // CORRECTED: Passing exactly 8 arguments matching your cyw43-pio 0.10.0 version constraints
        let pio_spi_bus = PioSpi::new(
            &pio.common,                      // 1. Common instance block reference
            sm0_block,                        // 2. Target State Machine handle token
            cyw43_pio::DEFAULT_CLOCK_DIVIDER, // 3. Divider frequency block
            pio_irq,                          // 4. PIO Irq vector mapping token
            mosi_pin,                         // 5. Hardware Output pin block (MOSI)
            miso_pin,                         // 6. DIO input pio pin trace handle (MISO)
            clk_pin,                          // 7. Clock pio pin trace handle (CLK)
            peripherals.DMA_CH4,              // 8. DMA channel assignment tracking asset
        );

        ExclusiveDevice::new(pio_spi_bus, cs_pin, embassy_time::Delay)
    };*/
    let aux_pio_spi = Err(AuxiliaryPioInitError::FeatureDisabled);

    (gyro_spi, blackbox_spi, aux_pio_spi, peripherals.FLASH)
}

/*{
    // --- INITIALIZE HARDWARE PERIPHERALS (RP2350 SPECIFIC) ---
    #[cfg(all(feature = "max7456", feature = "rp2350"))]
    let display_ref = {
        // Define SPI hardware transmission speed limits (e.g. 10MHz for MAX7456)
        let mut spi_config = Config::default();
        spi_config.frequency = 10_000_000;
        let spi_irq = interrupt::take!(SPI0);

        // Create the asynchronous SPI instance wrapping hardware SPI0 and DMA Channel 0
        let p = _peripherals;
        let spi = Spi::new(
            peripherals.SPI0,    // Hardware Peripheral Identifier
            peripherals.PIN_18,  // CLK Pin
            peripherals.PIN_19,  // TX (MOSI) Pin
            peripherals.PIN_16,  // RX (MISO) Pin
            peripherals.DMA_CH0, // TX DMA Channel assignment
            peripherals.DMA_CH1, // RX DMA Channel assignment
            spi_irq,
            spi_config,
        );

        // Leak to a safe static reference for the tasks
        let static_spi = SPI_DEVICE_CELL.init(spi);
        let display = DisplayPortMax7456::new(static_spi);

        DISPLAY_PORT_MUTEX_CELL.init(Mutex::new(display))
    };
    (gyro_spi, blackbox_spi)
}*/
