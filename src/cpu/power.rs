/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * power.rs
 * Reboot and shutdown
 */

use core::arch::asm;
use core::ptr::read_unaligned;

use crate::cpu::interrupts;
use crate::io::port::{write16, writeport};

fn u16at(ptr: *const u8, off: usize) -> u16 {
    unsafe { read_unaligned(ptr.add(off) as *const u16) }
}

fn u32at(ptr: *const u8, off: usize) -> u32 {
    unsafe { read_unaligned(ptr.add(off) as *const u32) }
}

fn u64at(ptr: *const u8, off: usize) -> u64 {
    unsafe { read_unaligned(ptr.add(off) as *const u64) }
}

fn same(ptr: *const u8, name: &[u8]) -> bool {
    for (i, value) in name.iter().enumerate() {
        if unsafe { *ptr.add(i) } != *value {
            return false;
        }
    }

    true
}

fn checksum(ptr: *const u8, len: usize) -> bool {
    let mut sum = 0u8;

    for i in 0..len {
        sum = sum.wrapping_add(unsafe { *ptr.add(i) });
    }

    sum == 0
}

// find the acpi root pointer
fn rsdp() -> *const u8 {
    let ebda = unsafe { read_unaligned(0x40E as *const u16) } as usize;

    if ebda != 0 {
        let base = ebda << 4;
        let end = (base + 1024).min(0xA0000);
        let mut addr = base;

        while addr + 20 <= end {
            let ptr = addr as *const u8;

            if same(ptr, b"RSD PTR ") && checksum(ptr, 20) {
                return ptr;
            }

            addr += 16;
        }
    }

    let mut addr = 0xE0000;

    while addr + 20 <= 0x100000 {
        let ptr = addr as *const u8;

        if same(ptr, b"RSD PTR ") && checksum(ptr, 20) {
            return ptr;
        }

        addr += 16;
    }

    core::ptr::null()
}

fn table(root: *const u8) -> (*const u8, usize) {
    let rev = unsafe { *root.add(15) };

    if rev >= 2 {
        let addr = u64at(root, 24);

        if addr != 0 {
            return (addr as *const u8, 8);
        }
    }

    (u32at(root, 16) as *const u8, 4)
}

// find the fixed acpi table
fn fadt() -> *const u8 {
    let root = rsdp();

    if root.is_null() {
        return core::ptr::null();
    }

    let (tab, step) = table(root);

    if tab.is_null() {
        return core::ptr::null();
    }

    let len = u32at(tab, 4) as usize;
    let mut off = 36;

    while off + step <= len {
        let addr = if step == 8 {
            u64at(tab, off)
        } else {
            u32at(tab, off) as u64
        };

        if addr != 0 {
            let ptr = addr as *const u8;

            if same(ptr, b"FACP") {
                return ptr;
            }
        }

        off += step;
    }

    core::ptr::null()
}

fn acpi() -> bool {
    let f = fadt();

    if f.is_null() {
        return false;
    }

    if u32at(f, 4) < 70 {
        return false;
    }

    let pm1a = u16at(f, 64);
    let pm1b = u16at(f, 68);

    if pm1a == 0 {
        return false;
    }

    // most boards use 5 for soft off
    let value = (5 << 10) | (1 << 13);

    write16(pm1a, value);

    if pm1b != 0 {
        write16(pm1b, value);
    }

    true
}

pub fn reboot() -> ! {
    interrupts::disable();

    // pulse the reset line of the keyboard controller
    writeport(0x64, 0xFE);

    writeport(0xCF9, 0x06);

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

pub fn shutdown() -> ! {
    interrupts::disable();

    acpi();

    // fallback for qemu and bochs
    write16(0x604, 0x2000);
    write16(0xB004, 0x2000);

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
