/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * keyboard.rs
 * Ps2 keyboard
 */

use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

const SIZE: usize = 16;

static BUF: [AtomicU8; SIZE] = [const { AtomicU8::new(0) }; SIZE];
static HEAD: AtomicUsize = AtomicUsize::new(0);
static TAIL: AtomicUsize = AtomicUsize::new(0);

// add a scancode
pub fn push(key: u8) {
    let head = HEAD.load(Ordering::Relaxed);
    let next = (head + 1) % SIZE;

    if next == TAIL.load(Ordering::Acquire) {
        return;
    }

    BUF[head].store(key, Ordering::Relaxed);
    HEAD.store(next, Ordering::Release);
}

// scancode to char
fn key(code: u8) -> Option<u8> {
    match code {
        0x02..=0x0B => Some(b"1234567890"[(code - 0x02) as usize]),
        0x0C => Some(b'-'),
        0x0D => Some(b'='),
        0x0E => Some(b'\x08'),
        0x10..=0x19 => Some(b"qwertyuiop"[(code - 0x10) as usize]),
        0x1A => Some(b'['),
        0x1B => Some(b']'),
        0x1C => Some(b'\n'),
        0x1E..=0x26 => Some(b"asdfghjkl"[(code - 0x1E) as usize]),
        0x27 => Some(b';'),
        0x28 => Some(b'\''),
        0x29 => Some(b'`'),
        0x2B => Some(b'\\'),
        0x2C..=0x32 => Some(b"zxcvbnm"[(code - 0x2C) as usize]),
        0x33 => Some(b','),
        0x34 => Some(b'.'),
        0x35 => Some(b'/'),
        0x39 => Some(b' '),
        _ => None,
    }
}

#[allow(dead_code)]
// get the next key
pub fn nextk() -> Option<u8> {
    loop {
        let tail = TAIL.load(Ordering::Relaxed);

        if tail == HEAD.load(Ordering::Acquire) {
            return None;
        }

        let code = BUF[tail].load(Ordering::Relaxed);
        TAIL.store((tail + 1) % SIZE, Ordering::Release);

        if let Some(ch) = key(code) {
            return Some(ch);
        }
    }
}
