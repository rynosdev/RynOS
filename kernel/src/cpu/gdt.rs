/*
 * gdt.rs
 * Copyright (c) RynOS 2026
 * Flat ring 0 GDT. Code and data, nothing else.
 */

static mut GDT: [u64; 3] = [
    0,
    0x00209a0000000000, // code
    0x0000920000000000, // data
];

#[repr(packed)]
// lgdt reads this through its memory operand so the compiler cannot see it
#[allow(dead_code)]
struct GdtPtr {
    limit: u16,
    base: u64,
}

const CODE_SEL: u16 = 8;
const DATA_SEL: u16 = 16;

pub const KERNEL_CODE: u16 = CODE_SEL;

pub fn load() {
    unsafe {
        let ptr = GdtPtr {
            limit: (core::mem::size_of_val(&GDT) - 1) as u16,
            base: &GDT as *const u64 as u64,
        };
        core::arch::asm!("lgdt [{}]", in(reg) &ptr);

        // reload cs with a far return
        core::arch::asm!(
            "push {sel}",
            "lea rax, [rip + 2f]",
            "push rax",
            "retfq",
            "2:",
            sel = in(reg) CODE_SEL as u64,
            out("rax") _,
        );

        core::arch::asm!(
            "mov ax, {v}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            v = const DATA_SEL,
            options(nostack),
        );
    }
}
