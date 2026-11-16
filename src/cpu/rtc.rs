/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * rtc.rs
 * Real time clock
 */

use crate::io::port::{readport, writeport};

const RTC: u16 = 0x70;
const DATA: u16 = 0x71;

#[derive(Clone, Copy, PartialEq, Eq)]
// current time
pub struct Time {
    pub day: u8,
    pub hour: u8,
    pub min: u8,
}

// read one cmos register
fn reg(idx: u8) -> u8 {
    writeport(RTC, idx);
    readport(DATA)
}

// bcd to binary
fn bcd(value: u8) -> u8 {
    (value & 0x0F) + (value >> 4) * 10
}

// day of week from date
fn weekday(year: u32, month: u32, day: u32) -> u8 {
    let add = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let month = month.clamp(1, 12);
    let day = day.clamp(1, 31);
    let mut y = year;

    if month < 3 {
        y -= 1;
    }

    let w = (y + y / 4 - y / 100 + y / 400 + add[(month - 1) as usize] + day) % 7;
    (w + 1) as u8
}

// short day name
pub fn name(day: u8) -> &'static str {
    match day {
        1 => "Sun",
        2 => "Mon",
        3 => "Tue",
        4 => "Wed",
        5 => "Thu",
        6 => "Fri",
        7 => "Sat",
        _ => "???",
    }
}

// read the clock
pub fn now() -> Time {
    while reg(0x0A) & 0x80 != 0 {}

    let mut min = reg(0x02);
    let mut hour = reg(0x04);
    let mut mday = reg(0x07);
    let mut mon = reg(0x08);
    let mut year = reg(0x09);

    while reg(0x0A) & 0x80 != 0 {}

    let mode = reg(0x0B);

    if mode & 0x04 == 0 {
        min = bcd(min);
        mday = bcd(mday);
        mon = bcd(mon);
        year = bcd(year);

        let pm = hour & 0x80 != 0;
        hour = bcd(hour & 0x7F);

        if mode & 0x02 == 0 {
            if hour == 12 {
                hour = 0;
            }

            if pm {
                hour += 12;
            }
        }
    } else if mode & 0x02 == 0 {
        let pm = hour & 0x80 != 0;
        hour &= 0x7F;

        if hour == 12 {
            hour = 0;
        }

        if pm {
            hour += 12;
        }
    }

    hour %= 12;

    if hour == 0 {
        hour = 12;
    }

    let day = weekday(2000 + year as u32, mon as u32, mday as u32);

    Time { day, hour, min }
}
