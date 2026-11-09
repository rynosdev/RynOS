/*
 * idt.rs
 * Copyright (c) RynOS 2026
 * Interrupt table built by hand, no crate help.
 */

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Gate {
    offset_low: u16,
    selector: u16,
    bits: u8,
    attr: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

impl Gate {
    pub const EMPTY: Gate = Gate {
        offset_low: 0,
        selector: 0,
        bits: 0,
        attr: 0,
        offset_mid: 0,
        offset_high: 0,
        zero: 0,
    };

    pub fn set(&mut self, addr: usize, attr: u8) {
        self.offset_low = addr as u16;
        self.offset_mid = (addr >> 16) as u16;
        self.offset_high = (addr >> 32) as u32;
        self.selector = crate::cpu::gdt::KERNEL_CODE;
        self.bits = 0;
        self.attr = attr;
    }
}

pub static mut IDT: [Gate; 256] = [Gate::EMPTY; 256];

#[repr(packed)]
// lidt also reads this memory directly
#[allow(dead_code)]
struct IdtPtr {
    limit: u16,
    base: u64,
}

// 0x8e: present, ring 0, interrupt gate
pub fn set_gate(v: u8, handler: usize) {
    unsafe {
        IDT[v as usize].set(handler, 0x8e);
    }
}

pub fn load() {
    unsafe {
        let ptr = IdtPtr {
            limit: (core::mem::size_of_val(&IDT) - 1) as u16,
            base: &IDT as *const Gate as u64,
        };
        core::arch::asm!("lidt [{}]", in(reg) &ptr);
    }
}
