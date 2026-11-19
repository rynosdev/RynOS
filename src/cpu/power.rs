/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * power.rs
 * Reboot and shutdown
 */

use core::arch::asm;
use core::ptr::read_unaligned;
use core::sync::atomic::{AtomicU16, Ordering};

use crate::cpu::interrupts;
use crate::io::port::{read16, readport, write16, writeport};

// event blocks of the two pm1 chips
static PM1A: AtomicU16 = AtomicU16::new(0);
static PM1B: AtomicU16 = AtomicU16::new(0);

// empty idt for the last reset
static ZERO: [u8; 10] = [0; 10];

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

// enable acpi mode and the power button event
pub fn init() {
    let f = fadt();

    if f.is_null() || u32at(f, 4) < 70 {
        return;
    }

    let evta = u16at(f, 56);
    let evtb = u16at(f, 60);
    let cnta = u16at(f, 64);
    let cntb = u16at(f, 68);

    // switch the chipset to acpi mode
    if cnta != 0 {
        write16(cnta, read16(cnta) | 1);
    }

    if cntb != 0 {
        write16(cntb, read16(cntb) | 1);
    }

    // let the power button raise an event
    if evta != 0 {
        write16(evta + 2, read16(evta + 2) | (1 << 8));
        PM1A.store(evta, Ordering::Release);
    }

    if evtb != 0 {
        write16(evtb + 2, read16(evtb + 2) | (1 << 8));
        PM1B.store(evtb, Ordering::Release);
    }
}

// true if the power button was pressed
pub fn poll() -> bool {
    let a = PM1A.load(Ordering::Acquire);
    let b = PM1B.load(Ordering::Acquire);

    if a != 0 && read16(a) & (1 << 8) != 0 {
        write16(a, 1 << 8);
        return true;
    }

    if b != 0 && read16(b) & (1 << 8) != 0 {
        write16(b, 1 << 8);
        return true;
    }

    false
}

fn acpi() -> bool {
    let f = fadt();

    if f.is_null() || u32at(f, 4) < 70 {
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

// try the reset register from the acpi table
fn acpireset() -> bool {
    let f = fadt();

    if f.is_null() || u32at(f, 4) < 129 {
        return false;
    }

    // the flag says the reset register is valid
    if u32at(f, 112) & (1 << 10) == 0 {
        return false;
    }

    let space = unsafe { *f.add(116) };
    let addr = u64at(f, 120);
    let value = unsafe { *f.add(128) };

    if addr == 0 {
        return false;
    }

    match space {
        0 => unsafe {
            (addr as *mut u8).write_volatile(value);
        },
        1 => writeport(addr as u16, value),
        _ => return false,
    }

    true
}

// give the reset time to fire
fn delay() {
    for _ in 0..1000 {
        readport(0x80);
    }
}

pub fn reboot() -> ! {
    interrupts::disable();

    if acpireset() {
        delay();
    }

    // chipset reset
    writeport(0xCF9, 0x06);
    delay();

    // pulse the reset line of the keyboard controller
    writeport(0x64, 0xFE);
    delay();

    // an empty idt faults the cpu and resets it
    unsafe {
        asm!("lidt [{}]", "int3", in(reg) &ZERO);
    }

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
