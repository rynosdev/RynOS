/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * cpu/idt.rs
 * Interrupt descriptor table
 */

use core::arch::asm;
use core::mem::size_of;
use core::ptr::{addr_of, addr_of_mut};

#[derive(Clone, Copy)]
#[repr(C)]
struct Entry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    flags: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

impl Entry {
    const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            flags: 0,
            offset_mid: 0,
            offset_high: 0,
            zero: 0,
        }
    }

    const fn gate(handler: u64) -> Self {
        Self {
            offset_low: handler as u16,
            selector: 0x08,
            ist: 0,
            flags: 0x8E,
            offset_mid: (handler >> 16) as u16,
            offset_high: (handler >> 32) as u32,
            zero: 0,
        }
    }
}

#[repr(C, align(16))]
struct Table([Entry; 256]);

static mut IDT: Table = Table([Entry::missing(); 256]);

#[repr(C, packed)]
struct DescriptorPointer {
    limit: u16,
    base: u64,
}

unsafe extern "C" {
    fn timer_stub();
    fn keyboard_stub();
}

pub fn init() {
    let table = addr_of_mut!(IDT) as *mut Entry;

    unsafe {
        // timer and keyboard gates
        table.add(32).write(Entry::gate(timer_stub as *const () as u64));
        table.add(33).write(Entry::gate(keyboard_stub as *const () as u64));
    }

    let ptr = DescriptorPointer {
        limit: (size_of::<Table>() - 1) as u16,
        base: addr_of!(IDT) as u64,
    };

    unsafe {
        // load idt
        asm!("lidt [{}]", in(reg) &ptr, options(readonly, nostack, preserves_flags));
    }
}
