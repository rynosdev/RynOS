/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * main.rs
 * Kernel init
 */

#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate alloc;

mod cpu;
mod drivers;
mod fs;
mod io;
mod mm;
mod video;

use core::arch::asm;
use core::ptr::read_unaligned;

use cpu::{gdt, idt, interrupts, pit, power, rtc};
use drivers::ata::ata;
use drivers::ps2::mouse;
use fs::rfs::Rfs;
use video::{font, framebuffer, menu};

const BOOT: u32 = 0x36D7_6289;
const FB: u32 = 8;

// read a multiboot number
fn u32at(ptr: *const u8, off: usize) -> u32 {
    unsafe { read_unaligned(ptr.add(off) as *const u32) }
}

// read a multiboot number
fn u64at(ptr: *const u8, off: usize) -> u64 {
    unsafe { read_unaligned(ptr.add(off) as *const u64) }
}

// find the framebuffer tag
fn video(info: u64) -> bool {
    if info == 0 {
        return false;
    }

    let ptr = info as *const u8;
    let total = u32at(ptr, 0);
    let mut off = 8;

    while off + 8 <= total as usize {
        let kind = u32at(ptr, off);
        let size = u32at(ptr, off + 4) as usize;

        if size < 8 {
            break;
        }

        if kind == FB && size >= 32 {
            let addr = u64at(ptr, off + 8);
            let pitch = u32at(ptr, off + 16);
            let width = u32at(ptr, off + 20);
            let height = u32at(ptr, off + 24);
            let bpp = unsafe { *ptr.add(off + 28) };
            let typ = unsafe { *ptr.add(off + 29) };

            if typ == 1 {
                framebuffer::init(addr, width, height, pitch, bpp);
                return true;
            }
        }

        off += (size + 7) & !7;
    }

    false
}

// mount or format the disk
fn disk() {
    match Rfs::mount() {
        Ok(_) => {}
        Err(()) => {
            let sectors = ata::sectors();

            if sectors != 0 && Rfs::blank().unwrap_or(false) && Rfs::format(sectors).is_ok() {
                let _ = Rfs::mount();
            }
        }
    }
}

// draw the desktop
fn desk() {
    let w = framebuffer::width();
    let h = framebuffer::height();
    let n = h.saturating_sub(1).max(1);

    for y in 0..h {
        let r = 0x3C - (0x3C - 0x0A) * y / n;
        let g = 0x7F - (0x7F - 0x1E) * y / n;
        let b = 0xC2 - (0xC2 - 0x3A) * y / n;
        framebuffer::rect(0, y, w, 1, (r << 16) | (g << 8) | b);
    }
}

// draw a part of the top bar
fn barpart(x: u32, w: u32) {
    for y in 0..24 {
        let r = 0xF9 - (0xF9 - 0xD3) * y / 23;
        let g = 0xFA - (0xFA - 0xD8) * y / 23;
        let b = 0xFC - (0xFC - 0xE0) * y / 23;
        framebuffer::rect(x, y, w, 1, (r << 16) | (g << 8) | b);
    }

    framebuffer::rect(x, 0, w, 1, 0xFFFFFF);
    framebuffer::rect(x, 22, w, 1, 0xF2F2F2);
    framebuffer::rect(x, 23, w, 1, 0x8A8A8A);
}

// draw the top bar
fn panel() {
    barpart(0, framebuffer::width());
    menu::titles();
}

// draw the clock text
fn clock(t: rtc::Time) {
    let mut buf = [0u8; 9];
    let day = rtc::name(t.day).as_bytes();

    buf[0] = day[0];
    buf[1] = day[1];
    buf[2] = day[2];
    buf[3] = b' ';

    let mut i = 4;

    if t.hour >= 10 {
        buf[i] = b'0' + t.hour / 10;
        i += 1;
    }

    buf[i] = b'0' + t.hour % 10;
    i += 1;
    buf[i] = b':';
    i += 1;
    buf[i] = b'0' + t.min / 10;
    i += 1;
    buf[i] = b'0' + t.min % 10;
    i += 1;

    let s = core::str::from_utf8(&buf[..i]).unwrap_or("");
    let x = framebuffer::width() - 10 - font::textw(s);

    font::text(x, 4, s, 0x161616);
}

#[unsafe(no_mangle)]
// kernel entry
pub extern "C" fn kmain(magic: u32, info: u64) -> ! {
    interrupts::disable();

    mm::heap::init();
    gdt::load();
    idt::init();
    interrupts::init();
    pit::init();

    if magic != BOOT || !video(info) {
        loop {
            unsafe {
                asm!("hlt");
            }
        }
    }

    let mut time = rtc::now();

    desk();
    panel();
    clock(time);

    mouse::init();
    mouse::update();

    ata::init();
    disk();

    interrupts::enable();

    let mut sec = 0;

    loop {
        unsafe {
            // wait for an interrupt
            asm!("hlt");
        }

        let moved = mouse::update();

        if moved && menu::isopen() {
            mouse::hide();
            let (x, y) = mouse::pos();
            menu::hover(x, y);
            mouse::show();
        }

        if let Some((x, y)) = mouse::click() {
            mouse::hide();
            let act = menu::click(x, y);

            match act {
                menu::Act::Shutdown => power::shutdown(),
                menu::Act::Reboot => power::reboot(),
                menu::Act::None => mouse::show(),
            }
        }

        let tick = pit::ticks() / 100;

        if tick != sec {
            sec = tick;
            let now = rtc::now();

            if now != time {
                time = now;
                let w = framebuffer::width();
                mouse::hide();
                barpart(w.saturating_sub(90), 90.min(w));
                clock(time);
                mouse::show();
            }
        }
    }
}

#[panic_handler]
// stop on a crash
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe {
            // stop the cpu
            asm!("cli; hlt");
        }
    }
}
