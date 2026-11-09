/*
 * keyboard.rs
 * Copyright (c) RynOS 2026
 * PS/2 keyboard on port 0x60. Scancodes set 1, ring buffer for keys.
 */

use crate::io::port::inb;

const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;

static mut BUFFER: [u8; 32] = [0; 32];
static mut READ: usize = 0;
static mut WRITE: usize = 0;

static mut SHIFT: bool = false;

// scancode set 1 make codes, 0 = no key
const MAP: [u8; 128] = build_map();
const MAP_SHIFT: [u8; 128] = build_map_shift();

const fn build_map() -> [u8; 128] {
    let mut m = [0u8; 128];
    m[0x02] = b'1';
    m[0x03] = b'2';
    m[0x04] = b'3';
    m[0x05] = b'4';
    m[0x06] = b'5';
    m[0x07] = b'6';
    m[0x08] = b'7';
    m[0x09] = b'8';
    m[0x0a] = b'9';
    m[0x0b] = b'0';
    m[0x0c] = b'-';
    m[0x0d] = b'=';
    m[0x0e] = 8; // backspace
    m[0x0f] = b'\t';
    m[0x10] = b'q';
    m[0x11] = b'w';
    m[0x12] = b'e';
    m[0x13] = b'r';
    m[0x14] = b't';
    m[0x15] = b'y';
    m[0x16] = b'u';
    m[0x17] = b'i';
    m[0x18] = b'o';
    m[0x19] = b'p';
    m[0x1a] = b'[';
    m[0x1b] = b']';
    m[0x1c] = b'\n';
    m[0x1e] = b'a';
    m[0x1f] = b's';
    m[0x20] = b'd';
    m[0x21] = b'f';
    m[0x22] = b'g';
    m[0x23] = b'h';
    m[0x24] = b'j';
    m[0x25] = b'k';
    m[0x26] = b'l';
    m[0x27] = b';';
    m[0x28] = b'\'';
    m[0x29] = b'`';
    m[0x2b] = b'\\';
    m[0x2c] = b'z';
    m[0x2d] = b'x';
    m[0x2e] = b'c';
    m[0x2f] = b'v';
    m[0x30] = b'b';
    m[0x31] = b'n';
    m[0x32] = b'm';
    m[0x33] = b',';
    m[0x34] = b'.';
    m[0x35] = b'/';
    m[0x39] = b' ';
    m
}

const fn build_map_shift() -> [u8; 128] {
    let mut m = MAP;
    m[0x02] = b'!';
    m[0x03] = b'@';
    m[0x04] = b'#';
    m[0x05] = b'$';
    m[0x06] = b'%';
    m[0x07] = b'^';
    m[0x08] = b'&';
    m[0x09] = b'*';
    m[0x0a] = b'(';
    m[0x0b] = b')';
    m[0x0c] = b'_';
    m[0x0d] = b'+';
    m[0x10] = b'Q';
    m[0x11] = b'W';
    m[0x12] = b'E';
    m[0x13] = b'R';
    m[0x14] = b'T';
    m[0x15] = b'Y';
    m[0x16] = b'U';
    m[0x17] = b'I';
    m[0x18] = b'O';
    m[0x19] = b'P';
    m[0x1a] = b'{';
    m[0x1b] = b'}';
    m[0x1e] = b'A';
    m[0x1f] = b'S';
    m[0x20] = b'D';
    m[0x21] = b'F';
    m[0x22] = b'G';
    m[0x23] = b'H';
    m[0x24] = b'J';
    m[0x25] = b'K';
    m[0x26] = b'L';
    m[0x27] = b':';
    m[0x28] = b'"';
    m[0x29] = b'~';
    m[0x2b] = b'|';
    m[0x2c] = b'Z';
    m[0x2d] = b'X';
    m[0x2e] = b'C';
    m[0x2f] = b'V';
    m[0x30] = b'B';
    m[0x31] = b'N';
    m[0x32] = b'M';
    m[0x33] = b'<';
    m[0x34] = b'>';
    m[0x35] = b'?';
    m
}

pub fn init() {
    unsafe {
        // flush pending bytes, just in case
        loop {
            if inb(STATUS_PORT) & 1 == 0 {
                break;
            }
            inb(DATA_PORT);
        }
    }
}

// called from the IRQ1 handler
pub fn irq_handler() {
    unsafe {
        let sc = inb(DATA_PORT);
        let is_break = sc & 0x80 != 0;
        let code = sc & 0x7f;
        match code {
            0x2a | 0x36 => SHIFT = !is_break,
            _ => {
                if !is_break {
                    let map = if SHIFT { &MAP_SHIFT } else { &MAP };
                    let c = map[code as usize];
                    if c != 0 {
                        push(c);
                    }
                }
            }
        }
    }
}

fn push(c: u8) {
    unsafe {
        let next = (WRITE + 1) % BUFFER.len();
        if next != READ {
            BUFFER[WRITE] = c;
            WRITE = next;
        }
        // if full, drop the key
    }
}

pub fn next_key() -> Option<u8> {
    unsafe {
        if READ == WRITE {
            return None;
        }
        let c = BUFFER[READ];
        READ = (READ + 1) % BUFFER.len();
        Some(c)
    }
}
