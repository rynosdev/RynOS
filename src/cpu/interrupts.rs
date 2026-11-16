/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * interrupts.rs
 * Interrupts and pic
 */

use core::arch::{asm, global_asm};

use crate::cpu::pit;
use crate::drivers::ps2::{keyboard, mouse};
use crate::io::port::{readport, writeport};

const PIC1: u16 = 0x20;
const PIC2: u16 = 0xA0;
const MASK1: u16 = 0x21;
const MASK2: u16 = 0xA1;

// interrupt entry stubs
global_asm!(
    r#"
.global timerstub
timerstub:
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
    call timerirq
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

.global keystub
keystub:
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
    call keyirq
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

.global mousestub
mousestub:
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
    call mouseirq
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

// small delay for the pic
fn iowait() {
    writeport(0x80, 0);
}

// send end of interrupt
fn eoi() {
    writeport(PIC1, 0x20);
}

// remap the pic
pub fn init() {
    writeport(PIC1, 0x11);
    iowait();
    writeport(PIC2, 0x11);
    iowait();
    writeport(MASK1, 0x20);
    iowait();
    writeport(MASK2, 0x28);
    iowait();
    writeport(MASK1, 0x04);
    iowait();
    writeport(MASK2, 0x02);
    iowait();
    writeport(MASK1, 0x01);
    iowait();
    writeport(MASK2, 0x01);
    iowait();

    writeport(MASK1, 0xF8);
    writeport(MASK2, 0xEF);
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
// timer interrupt
extern "C" fn timerirq() {
    pit::onirq();
    eoi();
}

#[unsafe(no_mangle)]
// keyboard interrupt
extern "C" fn keyirq() {
    let key = readport(0x60);
    keyboard::push(key);
    eoi();
}

#[unsafe(no_mangle)]
// mouse interrupt
extern "C" fn mouseirq() {
    mouse::onirq();
    writeport(PIC2, 0x20);
    eoi();
}
