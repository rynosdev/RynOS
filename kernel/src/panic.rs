/*
 * panic.rs
 * Copyright (c) RynOS 2026
 * Kernel panic handler. Prints the reason and stops the machine.
 */

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::serial_println!("PANIC: {}", info);
    loop {
        unsafe { core::arch::asm!("cli; hlt") }
    }
}
