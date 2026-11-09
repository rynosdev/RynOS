/*
 * serial.rs
 * Copyright (c) RynOS 2026
 * COM1 output. Boot logs and panic text go here.
 */

use crate::io::port::{inb, outb};
use core::fmt;

const COM1: u16 = 0x3f8;

pub fn init() {
    unsafe {
        // 38400 baud with divisor 3, 8N1, fifo on
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x80);
        outb(COM1, 0x03);
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03);
        outb(COM1 + 2, 0xc7);
        outb(COM1 + 4, 0x0b);
    }
}

pub fn write_str(s: &str) {
    for &byte in s.as_bytes() {
        unsafe {
            while inb(COM1 + 5) & 0x20 == 0 {}
            outb(COM1, byte);
        }
    }
}

pub struct Serial;

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_str(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! serial_println {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            writeln!($crate::io::serial::Serial, $($arg)*).ok();
        }
    };
}
