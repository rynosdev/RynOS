/*
 * interrupts.rs
 * Copyright (c) RynOS 2026
 * Exception handlers and the interrupt handler stubs. Remaps the PIC.
 */

use crate::cpu::idt;
use crate::io::port::{inb, io_wait, outb};

const ICW1_INIT: u8 = 0x11;
const ICW4_8086: u8 = 0x01;

// what iretq leaves on the stack
#[repr(C)]
pub struct StackFrame {
    rip: u64,
    cs: u64,
    flags: u64,
    rsp: u64,
    ss: u64,
}

macro_rules! fault {
    ($name:ident, $msg:expr) => {
        extern "x86-interrupt" fn $name(_f: &mut StackFrame) {
            panic!($msg);
        }
    };
    ($name:ident, $msg:expr, ec) => {
        extern "x86-interrupt" fn $name(_f: &mut StackFrame, _ec: u64) {
            panic!($msg);
        }
    };
}

// any CPU fault stops the kernel for now
fault!(divide_fault, "divide fault");
fault!(double_fault, "double fault", ec);
fault!(seg_fault, "segment not present", ec);
fault!(stack_fault, "stack fault", ec);
fault!(gp_fault, "general protection fault", ec);
fault!(page_fault, "page fault", ec);
fault!(ud_fault, "invalid opcode");
fault!(bad_entry, "unexpected exception");

extern "x86-interrupt" fn pit_tick(_f: &mut StackFrame) {
    send_eoi(0);
}

extern "x86-interrupt" fn keyboard_irq(_f: &mut StackFrame) {
    crate::drivers::ps2::keyboard::irq_handler();
    send_eoi(1);
}

pub fn send_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            outb(0xa0, 0x20);
        }
        outb(0x20, 0x20);
    }
}

fn set_irq_mask(port: u16, irq: u8, on: bool) {
    let mut v = unsafe { inb(port) };
    if on {
        v |= 1 << irq;
    } else {
        v &= !(1 << irq);
    }
    unsafe { outb(port, v) };
}

pub fn init() {
    unsafe {
        // move the PIC lines away from CPU exceptions (vectors 8..15)
        outb(0x20, ICW1_INIT);
        io_wait();
        outb(0xa0, ICW1_INIT);
        io_wait();
        outb(0x21, 32); // irq0 -> vector 32
        io_wait();
        outb(0xa1, 40); // irq8 -> vector 40
        io_wait();
        outb(0x21, 4); // slave sits on irq2
        io_wait();
        outb(0xa1, 2);
        io_wait();
        outb(0x21, ICW4_8086);
        io_wait();
        outb(0xa1, ICW4_8086);
        io_wait();

        // mask all lines then open the ones we use
        outb(0x21, 0xff);
        outb(0xa1, 0xff);
    }
    set_irq_mask(0x21, 0, false); // timer
    set_irq_mask(0x21, 1, false); // keyboard

    {
        let a = |v: u8, f: usize| idt::set_gate(v, f);
        a(0, divide_fault as usize);
        a(6, ud_fault as usize);
        a(8, double_fault as usize);
        a(10, seg_fault as usize);
        a(11, seg_fault as usize);
        a(12, stack_fault as usize);
        a(13, gp_fault as usize);
        a(14, page_fault as usize);
        a(3, bad_entry as usize);
        a(4, bad_entry as usize);
        a(5, bad_entry as usize);
        a(7, bad_entry as usize);
        a(16, bad_entry as usize);
        a(32, pit_tick as usize);
        a(33, keyboard_irq as usize);
    }

    idt::load();
}
