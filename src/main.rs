/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * main.rs
 * Kernel init
 */

#![no_std]
#![no_main]

mod boot;
mod cpu;
mod drivers;
mod io;

use core::arch::asm;

use cpu::{gdt, idt, interrupts, pit};
use drivers::ps2::keyboard;
use io::serial;

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    interrupts::disable();
    serial::init();

    serial_println!("rynos v0.0.1");

    gdt::load();
    serial_println!("gdt loaded");

    idt::init();
    serial_println!("idt loaded");

    interrupts::init();
    serial_println!("pic remapped");

    pit::init();
    serial_println!("pit started");

    interrupts::enable();

    loop {
        unsafe {
            // wait for an interrupt
            asm!("hlt");
        }

        while let Some(key) = keyboard::next_key() {
            serial_println!("key: {:02x}", key);
        }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial_println!("panic: {}", info);

    loop {
        unsafe {
            // stop the cpu
            asm!("hlt");
        }
    }
}
