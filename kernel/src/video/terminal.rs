/*
 * terminal.rs
 * Copyright (c) RynOS 2026
 * Text screen on the framebuffer. Grid, cursor and scrolling.
 *
 * Cell is 10 pixel wide so a glyph can grow to 9px and keep a gap.
 * Every set font bit becomes a 2x2 square, that is how we get the
 * heavy look of vga console text.
 */

use super::font::{Font, FONT_HEIGHT};
use super::framebuffer::{Framebuffer, BLACK, WHITE};
use core::fmt;

// glyph px + 2px thick stroke
const CELL_WIDTH: u32 = 10;

pub struct Terminal {
    fb: Framebuffer,
    cols: u32,
    rows: u32,
    x: u32,
    y: u32,
    font: Font,
}

// the one screen, used by the main loop and by panic
pub static mut TERMINAL: Option<Terminal> = None;

pub fn init(fb: Framebuffer) -> bool {
    unsafe {
        TERMINAL = Some(Terminal::new(fb));
    }
    true
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
        let cols = fb.width() / CELL_WIDTH;
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

    pub fn write_char(&mut self, c: char) {
        // clear the old cursor before anything else moves
        self.clear_cursor();
        match c {
            '\n' => {
                self.x = 0;
                self.y += 1;
            }
            '\r' => self.x = 0,
            '\x08' => {
                // backspace walks back one grid cell
                if self.x > 0 {
                    self.x -= 1;
                } else if self.y > 0 {
                    self.y -= 1;
                    self.x = self.cols - 1;
                }
                self.fb.fill_rect(
                    self.x * CELL_WIDTH,
                    self.y * FONT_HEIGHT,
                    CELL_WIDTH,
                    FONT_HEIGHT,
                    BLACK,
                );
            }
            '\t' => {
                // next tab stop, grid of 8
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
        let bx = self.x * CELL_WIDTH;
        let by = self.y * FONT_HEIGHT;
        // black cell first so the squares show up on clean ground
        self.fb.fill_rect(bx, by, CELL_WIDTH, FONT_HEIGHT, BLACK);
        for (row, bits) in glyph.iter().enumerate() {
            for bit in 0..8 {
                if bits & (0x80 >> bit) != 0 {
                    self.fb.fill_rect(bx + bit as u32, by + row as u32, 2, 2, WHITE);
                }
            }
        }
    }

    fn draw_cursor(&mut self) {
        let bx = self.x * CELL_WIDTH;
        let by = self.y * FONT_HEIGHT;
        self.fb.fill_rect(bx, by, 2, FONT_HEIGHT, WHITE);
    }

    fn clear_cursor(&mut self) {
        let bx = self.x * CELL_WIDTH;
        let by = self.y * FONT_HEIGHT;
        self.fb.fill_rect(bx, by, 2, FONT_HEIGHT, BLACK);
    }
}
