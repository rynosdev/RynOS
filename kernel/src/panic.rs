/*
 * panic.rs
 * Copyright (c) RynOS 2026
 * Kernel panic handler. Prints the reason and stops the machine.
 */

use core::fmt::Write;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::serial_println!("PANIC: {}", info);
    unsafe {
        if let Some(t) = crate::video::terminal::TERMINAL.as_mut() {
            let _ = writeln!(t, "\nPANIC: {}", info);
        }
    }
    loop {
        unsafe { core::arch::asm!("cli; hlt") }
    }
}
