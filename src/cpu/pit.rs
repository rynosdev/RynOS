/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * pit.rs
 * Timer
 */

use core::sync::atomic::{AtomicU64, Ordering};

use crate::io::port::writeport;

const CHANNEL0: u16 = 0x40;
const COMMAND: u16 = 0x43;

// ticks since boot
static TICKS: AtomicU64 = AtomicU64::new(0);

// start the timer
pub fn init() {
    let divisor: u16 = 11932;

    writeport(COMMAND, 0x36);
    writeport(CHANNEL0, (divisor & 0xFF) as u8);
    writeport(CHANNEL0, (divisor >> 8) as u8);
}

// read the tick count
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Acquire)
}

// count one tick
pub fn onirq() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}
