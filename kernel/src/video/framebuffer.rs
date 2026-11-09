/*
 * framebuffer.rs
 * Copyright (c) RynOS 2026
 * Linear framebuffer: pixels, rectangles, scrolling live here.
 */

pub struct Framebuffer {
    addr: *mut u8,
    width: u32,
    height: u32,
    pitch: u32,
    bpp: u8,
}

pub const BLACK: u32 = 0x00000000;
pub const WHITE: u32 = 0xd8d8d8;

impl Framebuffer {
    pub fn new(info: crate::boot::multiboot::FramebufferInfo) -> Option<Self> {
        // only 32bpp rgb for now
        if info.bpp != 32 {
            return None;
        }
        Some(Framebuffer {
            addr: info.addr as *mut u8,
            width: info.width,
            height: info.height,
            pitch: info.pitch,
            bpp: info.bpp,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let off = y * self.pitch + x * (self.bpp / 8) as u32;
        unsafe { *(self.addr.add(off as usize) as *mut u32) = color }
    }

    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        for row in y..y + h {
            for col in x..x + w {
                self.set_pixel(col, row, color);
            }
        }
    }

    pub fn clear(&mut self, color: u32) {
        self.fill_rect(0, 0, self.width, self.height, color);
    }

    // TEMP: draw a checker to test pixel addressing
    pub fn fb_check(&mut self) {
        for i in 0..64 {
            self.set_pixel(100 + i, 100, 0x00ff00);
            self.set_pixel(100, 100 + i, 0x0000ff);
        }
    }

    // one glyph row, 1 set bit = fg
    pub fn bit_row(&mut self, x: u32, y: u32, bits: u8, fg: u32, bg: u32) {
        for bit in 0..8 {
            let color = if bits & (0x80 >> bit) != 0 { fg } else { bg };
            self.set_pixel(x + bit as u32, y, color);
        }
    }

    // move screen content up by n pixel rows and clear the freed strip
    pub fn scroll_up(&mut self, lines: u32) {
        let shift = (lines * self.pitch) as usize;
        let copy_bytes = (self.height - lines) as usize * self.pitch as usize;
        let dst = self.addr as *mut u8;
        unsafe {
            let src = dst.add(shift);
            core::ptr::copy(src, dst, copy_bytes);
        }
        self.fill_rect(0, self.height - lines, self.width, lines, BLACK);
    }
}
