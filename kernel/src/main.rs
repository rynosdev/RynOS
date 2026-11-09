/*
 * main.rs
 * Copyright (c) RynOS 2026
 * Kernel entry point. Inits subsystems in order and runs the main loop.
 */

#![no_std]
#![no_main]

mod io;
mod panic;

#[no_mangle]
pub extern "C" fn kmain(mbi: u64) -> ! {
    io::serial::init();
    serial_println!("RynOS v0.0.1");
    serial_println!("multiboot info at {:#x}", mbi);

    loop {
        // interrupts are off this early, hlt just waits forever
        unsafe { core::arch::asm!("hlt") }
    }
}
