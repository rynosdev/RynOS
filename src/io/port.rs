/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * io/port.rs
 * Port io
 */

use core::arch::asm;

pub fn read_port(port: u16) -> u8 {
    let value: u8;
    unsafe {
        // port io
        asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    }
    value
}

pub fn write_port(port: u16, value: u8) {
    unsafe {
        // port io
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}
