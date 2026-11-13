/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * cpu/interrupts.rs
 * PIC and interrupt handlers
 */

use core::arch::{asm, global_asm};

use crate::cpu::pit;
use crate::drivers::ps2::keyboard;
use crate::io::port::{read_port, write_port};

const PIC1: u16 = 0x20;
const PIC2: u16 = 0xA0;
const PIC1_DATA: u16 = 0x21;
const PIC2_DATA: u16 = 0xA1;

global_asm!(
    r#"
.global timer_stub
timer_stub:
    push rax
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push rbp
    mov rbp, rsp
    and rsp, -16
    call timer_irq
    mov rsp, rbp
    pop rbp
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rax
    iretq

.global keyboard_stub
keyboard_stub:
    push rax
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push rbp
    mov rbp, rsp
    and rsp, -16
    call keyboard_irq
    mov rsp, rbp
    pop rbp
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rax
    iretq
"#
);

fn io_wait() {
    write_port(0x80, 0);
}

fn eoi() {
    write_port(PIC1, 0x20);
}

pub fn init() {
    write_port(PIC1, 0x11);
    io_wait();
    write_port(PIC2, 0x11);
    io_wait();
    write_port(PIC1_DATA, 0x20);
    io_wait();
    write_port(PIC2_DATA, 0x28);
    io_wait();
    write_port(PIC1_DATA, 0x04);
    io_wait();
    write_port(PIC2_DATA, 0x02);
    io_wait();
    write_port(PIC1_DATA, 0x01);
    io_wait();
    write_port(PIC2_DATA, 0x01);
    io_wait();

    write_port(PIC1_DATA, 0xFC);
    write_port(PIC2_DATA, 0xFF);
}

pub fn enable() {
    unsafe {
        // enable interrupts
        asm!("sti", options(nomem, nostack, preserves_flags));
    }
}

pub fn disable() {
    unsafe {
        // disable interrupts
        asm!("cli", options(nomem, nostack, preserves_flags));
    }
}

#[unsafe(no_mangle)]
extern "C" fn timer_irq() {
    pit::on_irq();
    eoi();
}

#[unsafe(no_mangle)]
extern "C" fn keyboard_irq() {
    let key = read_port(0x60);
    keyboard::push(key);
    eoi();
}
