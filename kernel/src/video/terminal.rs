/*
 * terminal.rs
 * Copyright (c) RynOS 2026
 * Text terminal on the framebuffer. Char grid, cursor, scrolling.
 */

use super::font::{Font, FONT_HEIGHT, FONT_WIDTH};
use super::framebuffer::{Framebuffer, BLACK, WHITE};
use core::fmt;

pub struct Terminal {
    fb: Framebuffer,
    cols: u32,
    rows: u32,
    x: u32,
    y: u32,
    font: Font,
}

// single instance of the screen, used by main loop and panic
pub static mut TERMINAL: Option<Terminal> = None;

// init screen. returns false if no usable framebuffer.
pub fn init(fb: Framebuffer) -> bool {
    unsafe {
        TERMINAL = Some(Terminal::new(fb));
    }
    true
}

// print on screen without touching globals inside panic
pub fn print_str(s: &str) {
    unsafe {
        if let Some(t) = TERMINAL.as_mut() {
            t.write_str(s);
        }
    }
}

impl fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}

impl Terminal {
    pub fn new(mut fb: Framebuffer) -> Self {
        let cols = fb.width() / FONT_WIDTH;
        let rows = fb.height() / FONT_HEIGHT;
        fb.clear(BLACK);
        Terminal {
            fb,
            cols,
            rows,
            x: 0,
            y: 0,
            font: Font,
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }

    pub fn write_char(&mut self, c: char) {
        // hide old cursor line while printing
        self.clear_cursor();
        match c {
            '\n' => {
                self.x = 0;
                self.y += 1;
            }
            '\r' => self.x = 0,
            '\x08' => {
                // backspace: step back one cell
                if self.x > 0 {
                    self.x -= 1;
                } else if self.y > 0 {
                    self.y -= 1;
                    self.x = self.cols - 1;
                }
                self.fb.fill_rect(
                    self.x * FONT_WIDTH,
                    self.y * FONT_HEIGHT,
                    FONT_WIDTH,
                    FONT_HEIGHT,
                    BLACK,
                );
            }
            '\t' => {
                // next tab stop of 8 chars
                self.x = (self.x + 8) & !(8 - 1);
                if self.x >= self.cols {
                    self.x = 0;
                    self.y += 1;
                }
            }
            c if (c as u32) < 128 => {
                self.draw_char(c as u8);
                self.x += 1;
                if self.x >= self.cols {
                    self.x = 0;
                    self.y += 1;
                }
            }
            _ => {}
        }
        if self.y >= self.rows {
            self.newline();
        }
        self.draw_cursor();
    }

    fn newline(&mut self) {
        if self.y >= self.rows {
            self.fb.scroll_up(FONT_HEIGHT);
            self.y = self.rows - 1;
        }
        self.x = 0;
    }

    fn draw_char(&mut self, c: u8) {
        let glyph = self.font.glyph(c);
        let bx = self.x * FONT_WIDTH;
        let by = self.y * FONT_HEIGHT;
        for (row, bits) in glyph.iter().enumerate() {
            self.fb.bit_row(bx, by + row as u32, *bits, WHITE, BLACK);
        }
    }

    fn draw_cursor(&mut self) {
        let bx = self.x * FONT_WIDTH;
        let by = self.y * FONT_HEIGHT;
        self.fb.fill_rect(bx, by + FONT_HEIGHT - 2, FONT_WIDTH, 2, WHITE);
    }

    fn clear_cursor(&mut self) {
        let bx = self.x * FONT_WIDTH;
        let by = self.y * FONT_HEIGHT;
        self.fb.fill_rect(bx, by + FONT_HEIGHT - 2, FONT_WIDTH, 2, BLACK);
    }
}
