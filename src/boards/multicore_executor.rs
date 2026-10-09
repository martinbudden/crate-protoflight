#![cfg(feature = "multicore")]

use core::cell::Cell;
use critical_section::Mutex;
use static_cell::StaticCell;

use embassy_executor::{Executor, SendSpawner};

use embassy_rp::{
    Peri,
    multicore::{Stack, spawn_core1},
    peripherals::CORE1,
};

pub fn start_core1_executor(core1: Peri<'static, CORE1>) -> SendSpawner {
    static EXECUTOR_CORE1: StaticCell<Executor> = StaticCell::new();
    static mut CORE1_STACK: Stack<4096> = Stack::new();
    static SPAWNER_SLOT: Mutex<Cell<Option<SendSpawner>>> = Mutex::new(Cell::new(None));

    // Spawn core1. The closure doesn't capture any local variables except the Send-safe `core1`.
    spawn_core1(core1, unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) }, move || {
        // Initialize the executor directly on Core 1
        let executor = EXECUTOR_CORE1.init(Executor::new());

        // Start the executor loop
        executor.run(|spawner| {
            // Convert to a SendSpawner and pass it back through our safe global slot
            critical_section::with(|cs| {
                SPAWNER_SLOT.borrow(cs).set(Some(spawner.make_send()));
            });
        });
    });

    // Back on Core 0, spin-wait until Core 1 writes the spawner into the slot
    loop {
        if let Some(spawner) = critical_section::with(|cs| SPAWNER_SLOT.borrow(cs).take()) {
            return spawner;
        }
    }
}
