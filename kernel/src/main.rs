/*
 * main.rs
 * Copyright (c) RynOS 2026
 * Boot the kernel. Init order is bottom up then the main loop.
 */

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod boot;
mod cpu;
mod drivers;
mod io;
mod panic;
mod video;

use crate::boot::multiboot;
use crate::video::{terminal, terminal::TERMINAL};

#[no_mangle]
pub extern "C" fn kmain(mbi: u64) -> ! {
    io::serial::init();
    serial_println!("RynOS v0.0.1");

    // video first so panics show up on screen
    let fb_info = match multiboot::find_framebuffer(mbi) {
        Some(info) => info,
        None => panic!("no framebuffer from bootloader"),
    };
    serial_println!(
        "framebuffer: {}x{}x{}, pitch {}, addr {:#x}",
        fb_info.width, fb_info.height, fb_info.bpp, fb_info.pitch, fb_info.addr
    );

    let fb = video::framebuffer::Framebuffer::new(fb_info)
        .unwrap_or_else(|| panic!("unsupported framebuffer mode"));
    terminal::init(fb);

    // tables and hardware
    cpu::gdt::load();
    cpu::interrupts::init();
    drivers::ps2::keyboard::init();
    unsafe { core::arch::asm!("sti") };

    serial_println!("booted");

    let t = match unsafe { TERMINAL.as_mut() } {
        Some(t) => t,
        None => panic!("terminal not ready"),
    };
    use core::fmt::Write;
    let _ = writeln!(t, "RynOS v0.0.1");
    let _ = writeln!(t, "type here: ");

    loop {
        if let Some(c) = drivers::ps2::keyboard::next_key() {
            t.write_char(c as char);
        } else {
            // sleep until the next interrupt
            unsafe { core::arch::asm!("hlt") }
        }
    }
}
