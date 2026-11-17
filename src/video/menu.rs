/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * menu.rs
 * Top menu
 */

use core::sync::atomic::{AtomicU32, Ordering};

use crate::video::{font, framebuffer};

const BAR: u32 = 24;
const IH: u32 = 22;
const PAD: u32 = 12;
const SW: usize = 160;
const SH: usize = 80;
const TW: usize = 80;
const TH: usize = 24;

#[derive(Clone, Copy)]
pub enum Act {
    None,
    Shutdown,
    Reboot,
}

struct Item {
    text: &'static str,
    act: Act,
}

struct Menu {
    title: &'static str,
    items: &'static [Item],
}

static OFF: [Item; 2] = [
    Item {
        text: "Shutdown",
        act: Act::Shutdown,
    },
    Item {
        text: "Reboot",
        act: Act::Reboot,
    },
];

static MENUS: [Menu; 1] = [Menu {
    title: "RynOS",
    items: &OFF,
}];

static OPEN: AtomicU32 = AtomicU32::new(0);
static HOVER: AtomicU32 = AtomicU32::new(0);
static SAVED: [AtomicU32; SW * SH] = [const { AtomicU32::new(0) }; SW * SH];
static TSAVED: [AtomicU32; TW * TH] = [const { AtomicU32::new(0) }; TW * TH];

fn titlex(index: usize) -> u32 {
    let mut x = 10;

    for i in 0..index {
        x += font::textw(MENUS[i].title) + 18;
    }

    x
}

fn boxx(index: usize) -> u32 {
    titlex(index).saturating_sub(5)
}

fn boxw(menu: &Menu) -> u32 {
    let mut w = font::textw(menu.title) + 24;

    for item in menu.items {
        w = w.max(font::textw(item.text) + PAD * 2 + 8);
    }

    w
}

fn boxh(menu: &Menu) -> u32 {
    menu.items.len() as u32 * IH + 8
}

fn openindex() -> Option<usize> {
    let value = OPEN.load(Ordering::Acquire);

    if value == 0 {
        None
    } else {
        Some(value as usize - 1)
    }
}

// save pixels under the dropdown
fn save(x: u32, y: u32, w: u32, h: u32) {
    let mut i = 0;

    for row in 0..h {
        for col in 0..w {
            let color = framebuffer::get(x + col, y + row);
            SAVED[i].store(color, Ordering::Relaxed);
            i += 1;
        }
    }
}

// put the old pixels back
fn restore(x: u32, y: u32, w: u32, h: u32) {
    let mut i = 0;

    for row in 0..h {
        for col in 0..w {
            let color = SAVED[i].load(Ordering::Relaxed);
            framebuffer::pixel(x + col, y + row, color);
            i += 1;
        }
    }
}

fn savetitle(x: u32, y: u32, w: u32, h: u32) {
    let mut i = 0;

    for row in 0..h {
        for col in 0..w {
            let color = framebuffer::get(x + col, y + row);
            TSAVED[i].store(color, Ordering::Relaxed);
            i += 1;
        }
    }
}

fn restoretitle(x: u32, y: u32, w: u32, h: u32) {
    let mut i = 0;

    for row in 0..h {
        for col in 0..w {
            let color = TSAVED[i].load(Ordering::Relaxed);
            framebuffer::pixel(x + col, y + row, color);
            i += 1;
        }
    }
}

fn mix(bg: u32, fg: u32, alpha: u32) -> u32 {
    let br = ((bg >> 16) & 0xFF) as u32;
    let bgc = ((bg >> 8) & 0xFF) as u32;
    let bb = (bg & 0xFF) as u32;
    let fr = ((fg >> 16) & 0xFF) as u32;
    let fgc = ((fg >> 8) & 0xFF) as u32;
    let fb = (fg & 0xFF) as u32;
    let r = (br * (255 - alpha) + fr * alpha) / 255;
    let g = (bgc * (255 - alpha) + fgc * alpha) / 255;
    let b = (bb * (255 - alpha) + fb * alpha) / 255;

    (r << 16) | (g << 8) | b
}

fn drawbox(x: u32, y: u32, w: u32, h: u32) {
    for row in 0..h {
        for col in 0..w {
            let i = (row * w + col) as usize;
            let bg = SAVED[i].load(Ordering::Relaxed);
            framebuffer::pixel(x + col, y + row, mix(bg, 0xF8F8F8, 190));
        }
    }
}

fn drawtitle(index: usize, active: bool) {
    let menu = &MENUS[index];
    let x = titlex(index);
    let w = font::textw(menu.title) + 10;

    if active {
        framebuffer::rect(x.saturating_sub(5), 0, w, BAR, 0x3B7BE0);
        font::text(x, 4, menu.title, 0xFFFFFF);
    } else {
        font::text(x, 4, menu.title, 0x161616);
    }
}

fn drawitems(index: usize) {
    let menu = &MENUS[index];
    let x = boxx(index);
    let w = boxw(menu);
    let hover = HOVER.load(Ordering::Acquire);

    for (i, item) in menu.items.iter().enumerate() {
        let y = BAR + 4 + i as u32 * IH;
        let ty = y + (IH - font::height()) / 2;

        if hover == i as u32 + 1 {
            framebuffer::rect(x, y, w, IH, 0x3B7BE0);
            font::text(x + PAD, ty, item.text, 0xFFFFFF);
        } else {
            font::text(x + PAD, ty, item.text, 0x161616);
        }
    }
}

fn open(index: usize) {
    let menu = &MENUS[index];
    let x = boxx(index);
    let w = boxw(menu);
    let h = boxh(menu);
    let tx = x.saturating_sub(1);
    let tw = font::textw(menu.title) + 12;

    savetitle(tx, 0, tw, BAR);
    save(x, BAR, w, h);
    OPEN.store(index as u32 + 1, Ordering::Release);
    HOVER.store(0, Ordering::Release);
    drawtitle(index, true);
    drawbox(x, BAR, w, h);
    drawitems(index);
}

fn close() {
    if let Some(index) = openindex() {
        let menu = &MENUS[index];
        let x = boxx(index);
        let w = boxw(menu);
        let h = boxh(menu);

        restore(x, BAR, w, h);
        restoretitle(x.saturating_sub(1), 0, font::textw(menu.title) + 12, BAR);
    }

    OPEN.store(0, Ordering::Release);
    HOVER.store(0, Ordering::Release);
    titles();
}

pub fn isopen() -> bool {
    openindex().is_some()
}

pub fn titles() {
    for i in 0..MENUS.len() {
        drawtitle(i, false);
    }
}

fn itemat(index: usize, x: i32, y: i32) -> Option<usize> {
    let menu = &MENUS[index];
    let left = boxx(index) as i32;
    let top = (BAR + 4) as i32;
    let w = boxw(menu) as i32;
    let h = (menu.items.len() as u32 * IH) as i32;

    if x < left || x >= left + w || y < top || y >= top + h {
        return None;
    }

    Some(((y - top) / IH as i32) as usize)
}

pub fn click(x: i32, y: i32) -> Act {
    if y >= 0 && y < BAR as i32 {
        for (i, menu) in MENUS.iter().enumerate() {
            let left = titlex(i) as i32;
            let right = left + font::textw(menu.title) as i32;

            if x >= left - 6 && x < right + 6 {
                if openindex() == Some(i) {
                    close();
                } else {
                    close();
                    open(i);
                }

                return Act::None;
            }
        }

        close();
        return Act::None;
    }

    if let Some(index) = openindex() {
        if let Some(item) = itemat(index, x, y) {
            let act = MENUS[index].items[item].act;
            close();
            return act;
        }
    }

    close();
    Act::None
}

pub fn hover(x: i32, y: i32) {
    if let Some(index) = openindex() {
        let next = itemat(index, x, y).map(|i| i as u32 + 1).unwrap_or(0);

        if HOVER.swap(next, Ordering::AcqRel) != next {
            drawbox(boxx(index), BAR, boxw(&MENUS[index]), boxh(&MENUS[index]));
            drawitems(index);
        }
    }
}
