/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * drivers/ps2/keyboard.rs
 * PS/2 keyboard
 */

use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

const SIZE: usize = 16;

static BUF: [AtomicU8; SIZE] = [const { AtomicU8::new(0) }; SIZE];
static HEAD: AtomicUsize = AtomicUsize::new(0);
static TAIL: AtomicUsize = AtomicUsize::new(0);

pub fn push(key: u8) {
    let head = HEAD.load(Ordering::Relaxed);
    let next = (head + 1) % SIZE;

    if next == TAIL.load(Ordering::Acquire) {
        return;
    }

    BUF[head].store(key, Ordering::Relaxed);
    HEAD.store(next, Ordering::Release);
}

pub fn next_key() -> Option<u8> {
    let tail = TAIL.load(Ordering::Relaxed);

    if tail == HEAD.load(Ordering::Acquire) {
        return None;
    }

    let key = BUF[tail].load(Ordering::Relaxed);
    TAIL.store((tail + 1) % SIZE, Ordering::Release);
    Some(key)
}
