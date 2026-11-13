/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * cpu/pit.rs
 * Programmable interval timer
 */

use core::sync::atomic::{AtomicU64, Ordering};

use crate::io::port::write_port;
use crate::serial_println;

const CHANNEL0: u16 = 0x40;
const COMMAND: u16 = 0x43;

static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    let divisor: u16 = 11932;

    write_port(COMMAND, 0x36);
    write_port(CHANNEL0, (divisor & 0xFF) as u8);
    write_port(CHANNEL0, (divisor >> 8) as u8);
}

pub fn on_irq() {
    let tick = TICKS.fetch_add(1, Ordering::Relaxed) + 1;

    if tick % 100 == 0 {
        serial_println!("tick");
    }
}
