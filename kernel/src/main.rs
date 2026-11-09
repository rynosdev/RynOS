/*
 * main.rs
 * Copyright (c) RynOS 2026
 * Kernel entry point. Inits subsystems in order and runs the main loop.
 */

#![no_std]
#![no_main]

mod boot;
mod io;
mod panic;

#[no_mangle]
pub extern "C" fn kmain(mbi: u64) -> ! {
    io::serial::init();
    io::serial::init();
    serial_println!("RynOS v0.0.1");

    if let Some(fb) = boot::multiboot::find_framebuffer(mbi) {
        serial_println!(
            "framebuffer: {}x{}x{}, pitch {}, addr {:#x}",
            fb.width, fb.height, fb.bpp, fb.pitch, fb.addr
        );
    } else {
        serial_println!("no framebuffer found");
    }

    loop {
        // interrupts are off this early, hlt just waits forever
        unsafe { core::arch::asm!("hlt") }
    }
}
