/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * mouse.rs
 * Ps2 mouse
 */

use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU8, AtomicUsize, Ordering};

use crate::io::port::{readport, writeport};
use crate::video::framebuffer;

const DATA: u16 = 0x60;
const STATUS: u16 = 0x64;

const CW: u32 = 12;
const CH: u32 = 19;

const SHAPE: [[u8; 12]; 19] = [
    *b"B...........",
    *b"BB..........",
    *b"BWB.........",
    *b"BWWB........",
    *b"BWWWB.......",
    *b"BWWWWB......",
    *b"BWWWWWB.....",
    *b"BWWWWWWB....",
    *b"BWWWWWWWB...",
    *b"BWWWWWWWWB..",
    *b"BWWWWWWWWWB.",
    *b"BWWWWWWBBBBB",
    *b"BWWWBWWB....",
    *b"BWWB.BWWB...",
    *b"BWB..BWWB...",
    *b"BB....BWWB..",
    *b"B.....BWWB..",
    *b".......BWWB.",
    *b".......BBBB.",
];

static PACKET: [AtomicU8; 3] = [const { AtomicU8::new(0) }; 3];
static INDEX: AtomicUsize = AtomicUsize::new(0);

static X: AtomicI32 = AtomicI32::new(0);
static Y: AtomicI32 = AtomicI32::new(0);
static OX: AtomicI32 = AtomicI32::new(-1);
static OY: AtomicI32 = AtomicI32::new(-1);
static MOVED: AtomicBool = AtomicBool::new(false);

// pixels under the cursor
static SAVED: [[AtomicU32; CW as usize]; CH as usize] =
    [const { [const { AtomicU32::new(0) }; CW as usize] }; CH as usize];

// wait for the controller
fn waitwrite() {
    for _ in 0..100_000 {
        if readport(STATUS) & 2 == 0 {
            return;
        }
    }
}

// wait for data
fn waitread() {
    for _ in 0..100_000 {
        if readport(STATUS) & 1 != 0 {
            return;
        }
    }
}

// send a controller command
fn command(value: u8) {
    waitwrite();
    writeport(STATUS, value);
}

// send a data byte
fn data(value: u8) {
    waitwrite();
    writeport(DATA, value);
}

// read one byte
fn read() -> u8 {
    waitread();
    readport(DATA)
}

// send a mouse command
fn send(value: u8) {
    command(0xD4);
    data(value);
    read();
}

// drop old bytes
fn flush() {
    for _ in 0..64 {
        if readport(STATUS) & 1 == 0 {
            break;
        }

        readport(DATA);
    }
}

// turn on the mouse
pub fn init() {
    flush();
    command(0xA8);
    command(0x20);

    let config = (read() | 0x02) & !0x20;
    command(0x60);
    data(config);

    send(0xF6);
    send(0xF4);

    let x = framebuffer::width() as i32 / 2;
    let y = framebuffer::height() as i32 / 2;

    X.store(x, Ordering::Release);
    Y.store(y, Ordering::Release);
    MOVED.store(true, Ordering::Release);
}

// read one mouse byte
pub fn onirq() {
    let value = readport(DATA);
    let index = INDEX.load(Ordering::Relaxed);

    if index == 0 && value & 0x08 == 0 {
        return;
    }

    PACKET[index].store(value, Ordering::Relaxed);

    if index < 2 {
        INDEX.store(index + 1, Ordering::Release);
        return;
    }

    INDEX.store(0, Ordering::Release);

    let flags = PACKET[0].load(Ordering::Relaxed);

    if flags & 0x80 != 0 || flags & 0xC0 != 0 {
        return;
    }

    let mut dx = PACKET[1].load(Ordering::Relaxed) as i32;
    let mut dy = PACKET[2].load(Ordering::Relaxed) as i32;

    if flags & 0x10 != 0 {
        dx -= 256;
    }

    if flags & 0x20 != 0 {
        dy -= 256;
    }

    let maxx = framebuffer::width().saturating_sub(CW) as i32;
    let maxy = framebuffer::height().saturating_sub(CH) as i32;

    let x = (X.load(Ordering::Acquire) + dx).clamp(0, maxx);
    let y = (Y.load(Ordering::Acquire) - dy).clamp(0, maxy);

    X.store(x, Ordering::Release);
    Y.store(y, Ordering::Release);
    MOVED.store(true, Ordering::Release);
}

// put the old pixels back
fn restore(x: i32, y: i32) {
    for row in 0..CH {
        for col in 0..CW {
            let color = SAVED[row as usize][col as usize].load(Ordering::Relaxed);
            framebuffer::pixel(x as u32 + col, y as u32 + row, color);
        }
    }
}

// save pixels and draw the cursor
fn draw(x: i32, y: i32) {
    for row in 0..CH {
        for col in 0..CW {
            let color = framebuffer::get(x as u32 + col, y as u32 + row);
            SAVED[row as usize][col as usize].store(color, Ordering::Relaxed);

            let value = SHAPE[row as usize][col as usize];

            if value == b'B' {
                framebuffer::pixel(x as u32 + col, y as u32 + row, 0x000000);
            } else if value == b'W' {
                framebuffer::pixel(x as u32 + col, y as u32 + row, 0xFFFFFF);
            }
        }
    }
}

// draw the cursor
pub fn show() {
    let x = X.load(Ordering::Acquire);
    let y = Y.load(Ordering::Acquire);

    draw(x, y);
    OX.store(x, Ordering::Release);
    OY.store(y, Ordering::Release);
}

// remove the cursor before a redraw
pub fn hide() {
    let x = OX.load(Ordering::Acquire);
    let y = OY.load(Ordering::Acquire);

    if x >= 0 && y >= 0 {
        restore(x, y);
    }

    OX.store(-1, Ordering::Release);
    OY.store(-1, Ordering::Release);
}

// move the cursor
pub fn update() {
    if !MOVED.swap(false, Ordering::AcqRel) {
        return;
    }

    let x = X.load(Ordering::Acquire);
    let y = Y.load(Ordering::Acquire);
    let ox = OX.load(Ordering::Acquire);
    let oy = OY.load(Ordering::Acquire);

    if ox >= 0 && oy >= 0 {
        restore(ox, oy);
    }

    draw(x, y);
    OX.store(x, Ordering::Release);
    OY.store(y, Ordering::Release);
}
