/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * idt.rs
 * Interrupt descriptor table
 */

use core::arch::asm;
use core::mem::size_of;
use core::ptr::{addr_of, addr_of_mut};

// one idt gate
#[derive(Clone, Copy)]
#[repr(C)]
struct Entry {
    low: u16,
    sel: u16,
    ist: u8,
    flags: u8,
    mid: u16,
    high: u32,
    zero: u32,
}

impl Entry {
    // empty gate
    const fn missing() -> Self {
        Self {
            low: 0,
            sel: 0,
            ist: 0,
            flags: 0,
            mid: 0,
            high: 0,
            zero: 0,
        }
    }

    // gate for a handler address
    const fn gate(addr: u64) -> Self {
        Self {
            low: addr as u16,
            sel: 0x08,
            ist: 0,
            flags: 0x8E,
            mid: (addr >> 16) as u16,
            high: (addr >> 32) as u32,
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
    fn timerstub();
    fn keystub();
    fn mousestub();
}

// load the interrupt table
pub fn init() {
    let table = addr_of_mut!(IDT) as *mut Entry;

    unsafe {
        // timer and keyboard gates
        table.add(32).write(Entry::gate(timerstub as *const () as u64));
        table.add(33).write(Entry::gate(keystub as *const () as u64));
        table.add(44).write(Entry::gate(mousestub as *const () as u64));
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
