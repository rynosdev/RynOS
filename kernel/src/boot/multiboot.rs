/*
 * multiboot.rs
 * Copyright (c) RynOS 2026
 * Reads multiboot2 boot information tags. Only what the kernel needs.
 */

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FramebufferTag {
    kind: u32,
    size: u32,
    addr: u64,
    pitch: u32,
    width: u32,
    height: u32,
    bpp: u8,
    fb_type: u8,
}

#[repr(C)]
struct Tag {
    kind: u32,
    size: u32,
}

const TAG_FRAMEBUFFER: u32 = 8;

pub struct FramebufferInfo {
    pub addr: u64,
    pub pitch: u32,
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
}

impl FramebufferTag {
    pub fn info(&self) -> FramebufferInfo {
        FramebufferInfo {
            addr: self.addr,
            pitch: self.pitch,
            width: self.width,
            height: self.height,
            bpp: self.bpp,
        }
    }
}

pub fn find_framebuffer(start: u64) -> Option<FramebufferInfo> {
    let mut off = 8;
    loop {
        let tag = (start + off) as *const Tag;
        let (kind, size) = unsafe { ((*tag).kind, (*tag).size) };
        if kind == 0 {
            return None;
        }
        if kind == TAG_FRAMEBUFFER {
            let fb = tag as *const FramebufferTag;
            return Some(unsafe { (*fb).info() });
        }
        off += size as u64;
        // tags are 8 byte aligned
        if off % 8 != 0 {
            off += 8 - off % 8;
        }
    }
}
