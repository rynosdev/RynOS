/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * io/serial.rs
 * COM1 serial output
 */

use core::fmt::{self, Write};

use crate::io::port::{read_port, write_port};

const COM1: u16 = 0x3F8;

pub fn init() {
    write_port(COM1 + 1, 0x00);
    write_port(COM1 + 3, 0x80);
    write_port(COM1 + 0, 0x01);
    write_port(COM1 + 1, 0x00);
    write_port(COM1 + 3, 0x03);
    write_port(COM1 + 2, 0xC7);
    write_port(COM1 + 4, 0x0B);
}

pub fn write_str(s: &str) {
    for byte in s.bytes() {
        while read_port(COM1 + 5) & 0x20 == 0 {}
        write_port(COM1, byte);
    }
}

struct Serial;

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_str(s);
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    let _ = Serial.write_fmt(args);
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::io::serial::print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! serial_println {
    () => {
        $crate::serial_print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::serial_print!("{}\n", format_args!($($arg)*))
    };
}
