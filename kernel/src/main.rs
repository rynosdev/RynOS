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
mod video;

use crate::boot::multiboot;
use crate::video::terminal;

#[no_mangle]
pub extern "C" fn kmain(mbi: u64) -> ! {
    io::serial::init();
    serial_println!("RynOS v0.0.1");

    let fb_info = match multiboot::find_framebuffer(mbi) {
        Some(info) => info,
        None => panic!("no framebuffer from bootloader"),
    };
    serial_println!(
        "framebuffer: {}x{}x{}, pitch {}, addr {:#x}",
        fb_info.width,
        fb_info.height,
        fb_info.bpp,
        fb_info.pitch,
        fb_info.addr
    );

    let fb = video::framebuffer::Framebuffer::new(fb_info)
        .unwrap_or_else(|| panic!("unsupported framebuffer mode"));
    terminal::init(fb);

    let t = match unsafe { terminal::TERMINAL.as_mut() } {
        Some(t) => t,
        None => panic!("terminal not ready"),
    };
    use core::fmt::Write;
    let _ = writeln!(t, "RynOS v0.0.1");
    let _ = writeln!(t, "ready. press keys when interrupts are on.");


    loop {
        // interrupts not up yet this early
        unsafe { core::arch::asm!("hlt") }
    }
}
