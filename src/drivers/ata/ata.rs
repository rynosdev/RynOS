/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * ata.rs
 * Ata disk
 */

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crate::io::port::{readport, read16, writeport, write16};

// ata ports
const DATA: u16 = 0x1F0;
const COUNT: u16 = 0x1F2;
const LBA0: u16 = 0x1F3;
const LBA1: u16 = 0x1F4;
const LBA2: u16 = 0x1F5;
const DRIVE: u16 = 0x1F6;
const STATUS: u16 = 0x1F7;
const CONTROL: u16 = 0x3F6;

const ERR: u8 = 0x01;
const DRQ: u8 = 0x08;
const BSY: u8 = 0x80;

const READ: u8 = 0x20;
const WRITE: u8 = 0x30;
const FLUSH: u8 = 0xE7;
const IDENTIFY: u8 = 0xEC;

static READY: AtomicBool = AtomicBool::new(false);
static SECTORS: AtomicU32 = AtomicU32::new(0);

// small delay after a port write
fn delay() {
    for _ in 0..4 {
        let _ = readport(STATUS);
    }
}

// wait until the disk is ready
fn waitbusy() -> Result<(), ()> {
    for _ in 0..1_000_000 {
        let status = readport(STATUS);

        if status == 0xFF {
            return Err(());
        }

        if status & BSY == 0 {
            return if status & ERR != 0 { Err(()) } else { Ok(()) };
        }
    }

    Err(())
}

// wait for data
fn waitdrq() -> Result<(), ()> {
    for _ in 0..1_000_000 {
        let status = readport(STATUS);

        if status == 0xFF {
            return Err(());
        }

        if status & BSY == 0 {
            if status & ERR != 0 {
                return Err(());
            }

            if status & DRQ != 0 {
                return Ok(());
            }
        }
    }

    Err(())
}

// select a sector
fn select(lba: u32) -> Result<(), ()> {
    writeport(DRIVE, 0xE0 | ((lba >> 24) as u8 & 0x0F));
    delay();
    waitbusy()
}

// find the disk
pub fn init() {
    READY.store(false, Ordering::Release);

    writeport(CONTROL, 0x04);
    delay();
    writeport(CONTROL, 0x00);
    delay();

    if waitbusy().is_err() {
        return;
    }

    writeport(DRIVE, 0xA0);
    delay();
    writeport(COUNT, 0);
    writeport(LBA0, 0);
    writeport(LBA1, 0);
    writeport(LBA2, 0);
    writeport(STATUS, IDENTIFY);
    delay();

    if waitdrq().is_err() {
        return;
    }

    let mut id = [0u16; 256];
    for word in id.iter_mut() {
        *word = read16(DATA);
    }

    let mut sectors = id[60] as u32 | (id[61] as u32) << 16;
    if sectors == 0 {
        return;
    }

    if sectors > 0x0FFF_FFFF {
        sectors = 0x0FFF_FFFF;
    }

    SECTORS.store(sectors, Ordering::Release);
    READY.store(true, Ordering::Release);
}

// disk size in sectors
pub fn sectors() -> u32 {
    SECTORS.load(Ordering::Acquire)
}

// read one sector
pub fn read(lba: u32, buf: &mut [u8; 512]) -> Result<(), ()> {
    if lba > 0x0FFF_FFFF || !READY.load(Ordering::Acquire) {
        return Err(());
    }

    select(lba)?;

    writeport(COUNT, 1);
    writeport(LBA0, lba as u8);
    writeport(LBA1, (lba >> 8) as u8);
    writeport(LBA2, (lba >> 16) as u8);
    writeport(STATUS, READ);
    delay();

    waitdrq()?;

    for i in 0..256 {
        let word = read16(DATA);
        buf[i * 2] = word as u8;
        buf[i * 2 + 1] = (word >> 8) as u8;
    }

    Ok(())
}

// write one sector
pub fn write(lba: u32, buf: &[u8; 512]) -> Result<(), ()> {
    if lba > 0x0FFF_FFFF || !READY.load(Ordering::Acquire) {
        return Err(());
    }

    select(lba)?;

    writeport(COUNT, 1);
    writeport(LBA0, lba as u8);
    writeport(LBA1, (lba >> 8) as u8);
    writeport(LBA2, (lba >> 16) as u8);
    writeport(STATUS, WRITE);
    delay();

    waitdrq()?;

    for i in 0..256 {
        let word = buf[i * 2] as u16 | (buf[i * 2 + 1] as u16) << 8;
        write16(DATA, word);
    }

    writeport(STATUS, FLUSH);
    delay();
    waitbusy()
}
