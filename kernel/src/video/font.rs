/*
 * font.rs
 * Copyright (c) RynOS 2026
 * 8x16 PSF2 console font. Ascii glyphs only.
 */

const FONT: &[u8] = include_bytes!("font.psf");

const HEIGHT: u32 = 16;
const CHARSIZE: u32 = 16;
const HEADER_SIZE: u32 = 32;

pub struct Font;

pub const FONT_HEIGHT: u32 = HEIGHT;
pub const FONT_WIDTH: u32 = 8;

impl Font {
    pub fn glyph(&self, c: u8) -> &[u8] {
        if c >= 128 {
            return &[0u8; CHARSIZE as usize];
        }
        let start = (HEADER_SIZE + (c as u32) * CHARSIZE) as usize;
        &FONT[start..start + CHARSIZE as usize]
    }
}
