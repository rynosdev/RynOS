/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * cpu/gdt.rs
 * Global descriptor table
 */

use core::arch::asm;
use core::mem::size_of;
use core::ptr::addr_of;

static GDT: [u64; 3] = [
    0,
    0x00AF9A000000FFFF,
    0x00CF92000000FFFF,
];

#[repr(C, packed)]
struct DescriptorPointer {
    limit: u16,
    base: u64,
}

pub fn load() {
    let ptr = DescriptorPointer {
        limit: (size_of::<[u64; 3]>() - 1) as u16,
        base: addr_of!(GDT) as u64,
    };

    unsafe {
        // load gdt and reload cs
        asm!("lgdt [{}]", in(reg) &ptr, options(readonly, nostack, preserves_flags));
        asm!(
            "push 0x08",
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",
            "2:",
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            "mov fs, ax",
            "mov gs, ax",
            tmp = out(reg) _,
        );
    }
}
