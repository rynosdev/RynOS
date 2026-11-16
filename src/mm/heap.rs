/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * heap.rs
 * Heap
 */

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{self, addr_of_mut};
use core::sync::atomic::{AtomicBool, Ordering};

// heap size and alignment
const HSIZE: usize = 4 * 1024 * 1024;
const HEAD: usize = 32;
const MINB: usize = 32;
const HALIGN: usize = 16;

// block header
#[repr(C)]
struct Head {
    size: usize,
    off: usize,
    next: *mut Head,
}

// simple spin lock
struct Lock(AtomicBool);

impl Lock {
    const fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    fn lock(&self) {
        while self.0.swap(true, Ordering::Acquire) {
            core::hint::spin_loop();
        }
    }

    fn unlock(&self) {
        self.0.store(false, Ordering::Release);
    }
}

static mut HEAP: [u8; HSIZE] = [0; HSIZE];
static mut FREE: *mut Head = ptr::null_mut();
static LOCK: Lock = Lock::new();

pub struct Heap;

#[global_allocator]
static ALLOC: Heap = Heap;

// round up to alignment
fn alignup(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

// set up the heap
pub fn init() {
    LOCK.lock();

    unsafe {
        // turn the static heap into one free block
        let base = addr_of_mut!(HEAP) as usize;
        let start = alignup(base, HALIGN);
        let head = start as *mut Head;

        (*head).size = base + HSIZE - start;
        (*head).off = 0;
        (*head).next = ptr::null_mut();

        FREE = head;
    }

    LOCK.unlock();
}

// find a free block
unsafe fn find(layout: Layout) -> *mut u8 {
    unsafe {
        // heap list pointers
        let size = alignup(layout.size().max(1), HALIGN);
        let align = layout.align().max(HALIGN);

        let mut prev: *mut Head = ptr::null_mut();
        let mut cur = FREE;

        while !cur.is_null() {
            let start = cur as usize;
            let bsize = (*cur).size;
            let bnext = (*cur).next;
            let user = start + HEAD;
            let off = alignup(user, align) - user;
            let need = off + HEAD + size;

            if bsize >= need {
                let rest = bsize - need;
                let place = (start + off) as *mut Head;

                if rest >= MINB {
                    let tail = (start + need) as *mut Head;

                    (*tail).size = rest;
                    (*tail).off = 0;
                    (*tail).next = bnext;

                    if prev.is_null() {
                        FREE = tail;
                    } else {
                        (*prev).next = tail;
                    }

                    (*place).size = need;
                } else {
                    if prev.is_null() {
                        FREE = bnext;
                    } else {
                        (*prev).next = bnext;
                    }

                    (*place).size = bsize;
                }

                (*place).off = off;
                return (place as usize + HEAD) as *mut u8;
            }

            prev = cur;
            cur = bnext;
        }

        ptr::null_mut()
    }
}

// return a block
unsafe fn release(ptr: *mut u8) {
    unsafe {
        // heap list pointers
        let head = (ptr as usize - HEAD) as *mut Head;
        let off = (*head).off;
        let size = (*head).size;
        let start = head as usize - off;

        let mut prev: *mut Head = ptr::null_mut();
        let mut cur = FREE;

        while !cur.is_null() && (cur as usize) < start {
            prev = cur;
            cur = (*cur).next;
        }

        let node = start as *mut Head;
        (*node).size = size;
        (*node).off = 0;
        (*node).next = cur;

        if prev.is_null() {
            FREE = node;
        } else {
            (*prev).next = node;
        }

        if !cur.is_null() && start + (*node).size == cur as usize {
            (*node).size += (*cur).size;
            (*node).next = (*cur).next;
        }

        if !prev.is_null() && (prev as usize) + (*prev).size == start {
            (*prev).size += (*node).size;
            (*prev).next = (*node).next;
        }
    }
}

unsafe impl GlobalAlloc for Heap {
    // allocate memory
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LOCK.lock();
        let ptr = unsafe { find(layout) };
        LOCK.unlock();
        ptr
    }

    // free memory
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        LOCK.lock();
        unsafe { release(ptr) };
        LOCK.unlock();
    }
}

// no memory left
#[alloc_error_handler]
fn oom(layout: Layout) -> ! {
    panic!("out of memory: {} bytes", layout.size());
}
