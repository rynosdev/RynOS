/*
 * port.rs
 * Copyright (c) RynOS 2026
 * x86 io port reads and writes.
 */

use core::arch::asm;

#[inline]
pub unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val)
}

#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    asm!("in al, dx", out("al") val, in("dx") port);
    val
}

// a write to port 0x80 gives the system a short pause
#[inline]
pub unsafe fn io_wait() {
    outb(0x80, 0);
}
