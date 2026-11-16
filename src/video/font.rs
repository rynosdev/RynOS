/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * font.rs
 * Font
 */

use crate::video::framebuffer;

// font data
const DATA: &[u8] = include_bytes!("../../assets/font/ttyp0-8x16b.psf2");

const MAGIC: u32 = 0x72B5_4A86;

// read a little endian number
fn u32at(off: usize) -> u32 {
    u32::from_le_bytes([
        DATA[off],
        DATA[off + 1],
        DATA[off + 2],
        DATA[off + 3],
    ])
}

// font header
struct Font {
    head: usize,
    count: usize,
    size: usize,
    width: usize,
    height: usize,
}

// read the font header
fn font() -> Font {
    if u32at(0) != MAGIC {
        return Font {
            head: 32,
            count: 0,
            size: 16,
            width: 8,
            height: 16,
        };
    }

    Font {
        head: u32at(8) as usize,
        count: u32at(16) as usize,
        size: u32at(20) as usize,
        height: u32at(24) as usize,
        width: u32at(28) as usize,
    }
}

// char width
pub fn width() -> u32 {
    font().width as u32
}

// char height
pub fn height() -> u32 {
    font().height as u32
}

// draw one char
pub fn draw(x: u32, y: u32, ch: u8, color: u32) {
    let f = font();
    let index = ch as usize;

    if index >= f.count {
        return;
    }

    let glyph = f.head + index * f.size;
    let rbytes = (f.width + 7) / 8;

    for row in 0..f.height {
        for col in 0..f.width {
            let byte = DATA[glyph + row * rbytes + col / 8];
            let bit = (byte >> (7 - col % 8)) & 1;

            if bit != 0 {
                framebuffer::pixel(x + col as u32, y + row as u32, color);
            }
        }
    }
}

// draw a string
pub fn text(x: u32, y: u32, s: &str, color: u32) {
    let mut pos = x;

    for ch in s.bytes() {
        draw(pos, y, ch, color);
        pos += width();
    }
}

// width of a string
pub fn textw(s: &str) -> u32 {
    s.len() as u32 * width()
}
