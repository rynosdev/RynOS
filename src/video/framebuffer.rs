/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * framebuffer.rs
 * Framebuffer
 */

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};

// screen info
static ADDR: AtomicU64 = AtomicU64::new(0);
static WIDTH: AtomicU32 = AtomicU32::new(0);
static HEIGHT: AtomicU32 = AtomicU32::new(0);
static PITCH: AtomicU32 = AtomicU32::new(0);
static BPP: AtomicU8 = AtomicU8::new(0);

// save the screen info
pub fn init(addr: u64, width: u32, height: u32, pitch: u32, bpp: u8) {
    ADDR.store(addr, Ordering::Release);
    WIDTH.store(width, Ordering::Release);
    HEIGHT.store(height, Ordering::Release);
    PITCH.store(pitch, Ordering::Release);
    BPP.store(bpp, Ordering::Release);
}

// screen width
pub fn width() -> u32 {
    WIDTH.load(Ordering::Acquire)
}

// screen height
pub fn height() -> u32 {
    HEIGHT.load(Ordering::Acquire)
}

// address of one pixel
fn base(x: u32, y: u32) -> u64 {
    let addr = ADDR.load(Ordering::Acquire);
    let pitch = PITCH.load(Ordering::Acquire);
    let bpp = BPP.load(Ordering::Acquire) as u64;

    addr + y as u64 * pitch as u64 + x as u64 * (bpp / 8)
}

// draw one pixel
pub fn pixel(x: u32, y: u32, color: u32) {
    if x >= width() || y >= height() {
        return;
    }

    let ptr = base(x, y) as *mut u8;
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;

    unsafe {
        match BPP.load(Ordering::Acquire) {
            32 => {
                let value = color | 0xFF00_0000;
                (ptr as *mut u32).write_volatile(value);
            }
            24 => {
                ptr.write_volatile(b);
                ptr.add(1).write_volatile(g);
                ptr.add(2).write_volatile(r);
            }
            16 => {
                let value = ((r as u16 >> 3) << 11) | ((g as u16 >> 2) << 5) | (b as u16 >> 3);
                (ptr as *mut u16).write_volatile(value);
            }
            _ => {}
        }
    }
}

// read one pixel
pub fn get(x: u32, y: u32) -> u32 {
    if x >= width() || y >= height() {
        return 0;
    }

    let ptr = base(x, y) as *const u8;

    unsafe {
        match BPP.load(Ordering::Acquire) {
            32 => {
                let value = (ptr as *const u32).read_volatile();
                value & 0x00FF_FFFF
            }
            24 => {
                let b = ptr.read_volatile() as u32;
                let g = ptr.add(1).read_volatile() as u32;
                let r = ptr.add(2).read_volatile() as u32;
                (r << 16) | (g << 8) | b
            }
            16 => {
                let value = (ptr as *const u16).read_volatile() as u32;
                let r = (value >> 11) & 0x1F;
                let g = (value >> 5) & 0x3F;
                let b = value & 0x1F;
                ((r << 3) | (r >> 2)) << 16 | ((g << 2) | (g >> 4)) << 8 | ((b << 3) | (b >> 2))
            }
            _ => 0,
        }
    }
}

// fill a rectangle
pub fn rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    let mut row = 0;

    while row < h {
        let mut col = 0;

        while col < w {
            pixel(x + col, y + row, color);
            col += 1;
        }

        row += 1;
    }
}

#[allow(dead_code)]
// fill the screen
pub fn clear(color: u32) {
    rect(0, 0, width(), height(), color);
}
