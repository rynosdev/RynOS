/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * rfs.rs
 * Ryn file system
 */

#![allow(dead_code)]

use core::str;
use core::sync::atomic::{AtomicU32, Ordering};

use crate::drivers::ata::ata;

const MAGIC: u64 = u64::from_le_bytes(*b"RYNFS001");
const VER: u32 = 1;

const SEC: usize = 512;
const INO: usize = 128;
const DE: usize = 64;
const DPS: usize = SEC / DE;
const NAMELEN: usize = 56;

const NDIR: usize = 12;
const NIND: usize = 128;

pub const FILE: u16 = 1;
pub const DIR: u16 = 2;

#[derive(Clone, Copy)]
// superblock
struct Super {
    blocks: u32,
    root: u32,
    inodes: u32,
    bmap: u32,
    imap: u32,
    itab: u32,
    data: u32,
}

#[derive(Clone, Copy)]
// inode
struct Inode {
    mode: u16,
    links: u16,
    size: u32,
    blocks: u32,
    direct: [u32; NDIR],
    indirect: u32,
    double: u32,
}

impl Inode {
    // empty inode
    fn new(mode: u16) -> Self {
        Self {
            mode,
            links: 0,
            size: 0,
            blocks: 0,
            direct: [0; NDIR],
            indirect: 0,
            double: 0,
        }
    }
}

// little endian helpers
fn get16(buf: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([buf[off], buf[off + 1]])
}

fn get32(buf: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
}

fn get64(buf: &[u8], off: usize) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&buf[off..off + 8]);
    u64::from_le_bytes(bytes)
}

fn put16(buf: &mut [u8], off: usize, value: u16) {
    buf[off..off + 2].copy_from_slice(&value.to_le_bytes());
}

fn put32(buf: &mut [u8], off: usize, value: u32) {
    buf[off..off + 4].copy_from_slice(&value.to_le_bytes());
}

fn put64(buf: &mut [u8], off: usize, value: u64) {
    buf[off..off + 8].copy_from_slice(&value.to_le_bytes());
}

// divide and round up
fn divup(value: u32, by: u32) -> u32 {
    (value + by - 1) / by
}

// place the fs parts
fn geom(blocks: u32) -> Option<Super> {
    if blocks < 64 {
        return None;
    }

    let mut inodes = blocks / 16;
    if inodes > 4096 {
        inodes = 4096;
    }
    if inodes < 64 {
        inodes = 64;
    }

    let bmap = 1;
    let imap = bmap + divup(blocks, 8 * SEC as u32);
    let itab = imap + divup(inodes, 8 * SEC as u32);
    let data = itab + divup(inodes * INO as u32, SEC as u32);

    if data.checked_add(8)? >= blocks {
        return None;
    }

    Some(Super {
        blocks,
        root: 1,
        inodes,
        bmap,
        imap,
        itab,
        data,
    })
}

// check a file name
fn okname(name: &str) -> bool {
    let bytes = name.as_bytes();

    !bytes.is_empty()
        && bytes.len() <= NAMELEN
        && !bytes.contains(&b'/')
        && name != "."
        && name != ".."
}

// write one dir entry
fn putent(buf: &mut [u8], slot: usize, ino: u32, typ: u16, name: &str) {
    let off = slot * DE;

    put32(buf, off, ino);
    put16(buf, off + 4, typ);
    put16(buf, off + 6, name.len() as u16);
    buf[off + 8..off + 8 + name.len()].copy_from_slice(name.as_bytes());
}

// set one bitmap bit
fn setbit(start: u32, count: u32, index: u32, value: bool) -> Result<(), ()> {
    if index >= count {
        return Err(());
    }

    let mut buf = [0u8; SEC];
    let lba = start + index / 4096;
    ata::read(lba, &mut buf)?;

    let byte = ((index % 4096) / 8) as usize;
    let mask = 1u8 << (index % 8);

    if value {
        buf[byte] |= mask;
    } else {
        buf[byte] &= !mask;
    }

    ata::write(lba, &buf)
}

// find a free bitmap bit
fn findbit(start: u32, from: u32, to: u32) -> Result<Option<u32>, ()> {
    let mut buf = [0u8; SEC];
    let mut lba = u32::MAX;

    for index in from..to {
        let sector = start + index / 4096;

        if sector != lba {
            ata::read(sector, &mut buf)?;
            lba = sector;
        }

        let byte = buf[((index % 4096) / 8) as usize];

        if byte & (1u8 << (index % 8)) == 0 {
            return Ok(Some(index));
        }
    }

    Ok(None)
}

// mark used blocks
fn mark(sup: &Super, from: u32, to: u32) -> Result<(), ()> {
    let mut buf = [0u8; SEC];
    let mut lba = u32::MAX;

    for index in from..to {
        let sector = sup.bmap + index / 4096;

        if sector != lba {
            if lba != u32::MAX {
                ata::write(lba, &buf)?;
            }

            ata::read(sector, &mut buf)?;
            lba = sector;
        }

        let byte = ((index % 4096) / 8) as usize;
        buf[byte] |= 1u8 << (index % 8);
    }

    if lba != u32::MAX {
        ata::write(lba, &buf)?;
    }

    Ok(())
}

// mounted file system
pub struct Rfs {
    sup: Super,
    hint: AtomicU32,
}

impl Rfs {
    // make a new file system
    pub fn format(blocks: u32) -> Result<(), ()> {
        let sup = geom(blocks).ok_or(())?;

        let mut buf = [0u8; SEC];
        put64(&mut buf, 0, MAGIC);
        put32(&mut buf, 8, VER);
        put32(&mut buf, 12, sup.blocks);
        put32(&mut buf, 16, sup.root);
        put32(&mut buf, 20, sup.inodes);
        put32(&mut buf, 24, sup.bmap);
        put32(&mut buf, 28, sup.imap);
        put32(&mut buf, 32, sup.itab);
        put32(&mut buf, 36, sup.data);
        put32(&mut buf, 40, sup.data);
        ata::write(0, &buf)?;

        let zero = [0u8; SEC];
        for lba in sup.bmap..sup.data {
            ata::write(lba, &zero)?;
        }

        mark(&sup, 0, sup.data)?;
        setbit(sup.imap, sup.inodes, 0, true)?;
        setbit(sup.imap, sup.inodes, 1, true)?;

        let fs = Self {
            sup,
            hint: AtomicU32::new(sup.data),
        };

        let block = fs.newblock()?;
        let mut root = Inode::new(DIR);
        root.links = 2;
        root.size = SEC as u32;
        root.blocks = 1;
        root.direct[0] = block;
        fs.wrino(sup.root, &root)?;

        let mut dir = [0u8; SEC];
        putent(&mut dir, 0, sup.root, DIR, ".");
        putent(&mut dir, 1, sup.root, DIR, "..");
        fs.wrsec(block, &dir)?;
        Ok(())
    }

    // read the superblock
    pub fn mount() -> Result<Self, ()> {
        let mut buf = [0u8; SEC];
        ata::read(0, &mut buf)?;

        if get64(&buf, 0) != MAGIC || get32(&buf, 8) != VER {
            return Err(());
        }

        let sup = Super {
            blocks: get32(&buf, 12),
            root: get32(&buf, 16),
            inodes: get32(&buf, 20),
            bmap: get32(&buf, 24),
            imap: get32(&buf, 28),
            itab: get32(&buf, 32),
            data: get32(&buf, 36),
        };

        let expect = geom(sup.blocks).ok_or(())?;

        if sup.root == 0
            || sup.root >= sup.inodes
            || sup.inodes != expect.inodes
            || sup.bmap != expect.bmap
            || sup.imap != expect.imap
            || sup.itab != expect.itab
            || sup.data != expect.data
        {
            return Err(());
        }

        let mut hint = get32(&buf, 40);
        if hint < sup.data || hint >= sup.blocks {
            hint = sup.data;
        }

        let fs = Self {
            sup,
            hint: AtomicU32::new(hint),
        };

        if fs.rdino(sup.root)?.mode != DIR {
            return Err(());
        }

        Ok(fs)
    }

    // check for a blank disk
    pub fn blank() -> Result<bool, ()> {
        let mut buf = [0u8; SEC];
        ata::read(0, &mut buf)?;
        Ok(buf.iter().all(|byte| *byte == 0))
    }

    // read and write one sector
    fn rdsec(&self, lba: u32, buf: &mut [u8; SEC]) -> Result<(), ()> {
        ata::read(lba, buf)
    }

    fn wrsec(&self, lba: u32, buf: &[u8; SEC]) -> Result<(), ()> {
        ata::write(lba, buf)
    }

    // read and write one inode
    fn rdino(&self, ino: u32) -> Result<Inode, ()> {
        if ino >= self.sup.inodes {
            return Err(());
        }

        let mut buf = [0u8; SEC];
        self.rdsec(self.sup.itab + ino / 4, &mut buf)?;

        let off = (ino % 4) as usize * INO;
        let mut direct = [0u32; NDIR];

        for (i, block) in direct.iter_mut().enumerate() {
            *block = get32(&buf, off + 12 + i * 4);
        }

        Ok(Inode {
            mode: get16(&buf, off),
            links: get16(&buf, off + 2),
            size: get32(&buf, off + 4),
            blocks: get32(&buf, off + 8),
            direct,
            indirect: get32(&buf, off + 60),
            double: get32(&buf, off + 64),
        })
    }

    fn wrino(&self, ino: u32, inode: &Inode) -> Result<(), ()> {
        if ino >= self.sup.inodes {
            return Err(());
        }

        let mut buf = [0u8; SEC];
        self.rdsec(self.sup.itab + ino / 4, &mut buf)?;

        let off = (ino % 4) as usize * INO;

        put16(&mut buf, off, inode.mode);
        put16(&mut buf, off + 2, inode.links);
        put32(&mut buf, off + 4, inode.size);
        put32(&mut buf, off + 8, inode.blocks);

        for (i, block) in inode.direct.iter().enumerate() {
            put32(&mut buf, off + 12 + i * 4, *block);
        }

        put32(&mut buf, off + 60, inode.indirect);
        put32(&mut buf, off + 64, inode.double);

        self.wrsec(self.sup.itab + ino / 4, &buf)
    }

    // alloc and free an inode
    fn newino(&self) -> Result<u32, ()> {
        let ino = findbit(self.sup.imap, 2, self.sup.inodes)?.ok_or(())?;

        setbit(self.sup.imap, self.sup.inodes, ino, true)?;
        self.wrino(ino, &Inode::new(0))?;
        Ok(ino)
    }

    fn delino(&self, ino: u32) -> Result<(), ()> {
        if ino < 2 {
            return Err(());
        }

        setbit(self.sup.imap, self.sup.inodes, ino, false)
    }

    // alloc and free a block
    fn newblock(&self) -> Result<u32, ()> {
        let from = self.hint.load(Ordering::Relaxed).max(self.sup.data);

        let block = match findbit(self.sup.bmap, from, self.sup.blocks)? {
            Some(block) => block,
            None => findbit(self.sup.bmap, self.sup.data, from)?
                .ok_or(())?,
        };

        setbit(self.sup.bmap, self.sup.blocks, block, true)?;

        let zero = [0u8; SEC];
        self.wrsec(block, &zero)?;

        self.hint.store(block + 1, Ordering::Relaxed);
        Ok(block)
    }

    fn delblock(&self, block: u32) -> Result<(), ()> {
        if block < self.sup.data || block >= self.sup.blocks {
            return Err(());
        }

        setbit(self.sup.bmap, self.sup.blocks, block, false)?;

        if block < self.hint.load(Ordering::Relaxed) {
            self.hint.store(block, Ordering::Relaxed);
        }

        Ok(())
    }

    // get a direct block
    fn getdir(&self, inode: &mut Inode, index: u32, alloc: bool) -> Result<u32, ()> {
        let slot = index as usize;

        if inode.direct[slot] == 0 {
            if !alloc {
                return Ok(0);
            }

            inode.direct[slot] = self.newblock()?;
            inode.blocks += 1;
        }

        Ok(inode.direct[slot])
    }

    // get an indirect block
    fn getind(&self, inode: &mut Inode, index: u32, alloc: bool) -> Result<u32, ()> {
        if inode.indirect == 0 {
            if !alloc {
                return Ok(0);
            }

            inode.indirect = self.newblock()?;
            inode.blocks += 1;
        }

        let mut one = [0u8; SEC];
        self.rdsec(inode.indirect, &mut one)?;

        let slot = (index - NDIR as u32) as usize;
        let mut block = get32(&one, slot * 4);

        if block == 0 {
            if !alloc {
                return Ok(0);
            }

            block = self.newblock()?;
            put32(&mut one, slot * 4, block);
            self.wrsec(inode.indirect, &one)?;
            inode.blocks += 1;
        }

        Ok(block)
    }

    // get a double indirect block
    fn getdbl(&self, inode: &mut Inode, index: u32, alloc: bool) -> Result<u32, ()> {
        let index = index - NDIR as u32 - NIND as u32;

        if index >= NIND as u32 * NIND as u32 {
            return Err(());
        }

        if inode.double == 0 {
            if !alloc {
                return Ok(0);
            }

            inode.double = self.newblock()?;
            inode.blocks += 1;
        }

        let mut two = [0u8; SEC];
        self.rdsec(inode.double, &mut two)?;

        let slot = (index / NIND as u32) as usize;
        let mut one = get32(&two, slot * 4);

        if one == 0 {
            if !alloc {
                return Ok(0);
            }

            one = self.newblock()?;
            put32(&mut two, slot * 4, one);
            self.wrsec(inode.double, &two)?;
            inode.blocks += 1;
        }

        let mut mid = [0u8; SEC];
        self.rdsec(one, &mut mid)?;

        let slot = (index % NIND as u32) as usize;
        let mut block = get32(&mid, slot * 4);

        if block == 0 {
            if !alloc {
                return Ok(0);
            }

            block = self.newblock()?;
            put32(&mut mid, slot * 4, block);
            self.wrsec(one, &mid)?;
            inode.blocks += 1;
        }

        Ok(block)
    }

    // get a file block
    fn getblock(&self, ino: u32, index: u32, alloc: bool) -> Result<u32, ()> {
        let mut inode = self.rdino(ino)?;
        let start = inode.blocks;

        let block = if (index as usize) < NDIR {
            self.getdir(&mut inode, index, alloc)?
        } else if index < NDIR as u32 + NIND as u32 {
            self.getind(&mut inode, index, alloc)?
        } else {
            self.getdbl(&mut inode, index, alloc)?
        };

        if inode.blocks != start {
            self.wrino(ino, &inode)?;
        }

        Ok(block)
    }

    // free an indirect block
    fn freeind(&self, block: u32) -> Result<(), ()> {
        let mut one = [0u8; SEC];
        self.rdsec(block, &mut one)?;

        for slot in 0..NIND {
            let data = get32(&one, slot * 4);

            if data != 0 {
                self.delblock(data)?;
            }
        }

        self.delblock(block)
    }

    // free a double indirect block
    fn freedbl(&self, block: u32) -> Result<(), ()> {
        let mut two = [0u8; SEC];
        self.rdsec(block, &mut two)?;

        for slot in 0..NIND {
            let one = get32(&two, slot * 4);

            if one != 0 {
                self.freeind(one)?;
            }
        }

        self.delblock(block)
    }

    // empty a file
    pub fn trunc(&self, ino: u32) -> Result<(), ()> {
        let mut inode = self.rdino(ino)?;

        for block in inode.direct.iter_mut() {
            if *block != 0 {
                self.delblock(*block)?;
                *block = 0;
            }
        }

        if inode.indirect != 0 {
            self.freeind(inode.indirect)?;
            inode.indirect = 0;
        }

        if inode.double != 0 {
            self.freedbl(inode.double)?;
            inode.double = 0;
        }

        inode.size = 0;
        inode.blocks = 0;
        self.wrino(ino, &inode)
    }

    // find a dir entry
    fn findent(&self, dir: u32, name: &str) -> Result<Option<u32>, ()> {
        let want = name.as_bytes();

        if want.is_empty() || want.len() > NAMELEN {
            return Err(());
        }

        let inode = self.rdino(dir)?;

        if inode.mode != DIR {
            return Err(());
        }

        let mut index = 0;

        while (index * SEC as u32) < inode.size {
            let block = self.getblock(dir, index, false)?;

            if block != 0 {
                let mut buf = [0u8; SEC];
                self.rdsec(block, &mut buf)?;

                for slot in 0..DPS {
                    let off = slot * DE;
                    let found = get32(&buf, off);
                    let len = get16(&buf, off + 6) as usize;

                    if found != 0
                        && len == want.len()
                        && &buf[off + 8..off + 8 + len] == want
                    {
                        return Ok(Some(found));
                    }
                }
            }

            index += 1;
        }

        Ok(None)
    }

    // walk a dir entry
    fn lookup(&self, dir: u32, name: &str) -> Result<u32, ()> {
        self.findent(dir, name)?.ok_or(())
    }

    // add a dir entry
    fn addent(&self, dir: u32, name: &str, ino: u32, typ: u16) -> Result<(), ()> {
        if !okname(name) {
            return Err(());
        }

        let inode = self.rdino(dir)?;

        if inode.mode != DIR {
            return Err(());
        }

        let mut index = 0;

        while (index * SEC as u32) < inode.size {
            let block = self.getblock(dir, index, false)?;

            if block != 0 {
                let mut buf = [0u8; SEC];
                self.rdsec(block, &mut buf)?;

                for slot in 0..DPS {
                    if get32(&buf, slot * DE) == 0 {
                        putent(&mut buf, slot, ino, typ, name);
                        self.wrsec(block, &buf)?;
                        return Ok(());
                    }
                }
            }

            index += 1;
        }

        let block = self.getblock(dir, index, true)?;
        let mut buf = [0u8; SEC];
        putent(&mut buf, 0, ino, typ, name);
        self.wrsec(block, &buf)?;

        let mut inode = self.rdino(dir)?;
        inode.size += SEC as u32;
        self.wrino(dir, &inode)
    }

    // remove a dir entry
    fn delent(&self, dir: u32, name: &str) -> Result<(), ()> {
        let want = name.as_bytes();
        let inode = self.rdino(dir)?;

        if inode.mode != DIR {
            return Err(());
        }

        let mut index = 0;

        while (index * SEC as u32) < inode.size {
            let block = self.getblock(dir, index, false)?;

            if block != 0 {
                let mut buf = [0u8; SEC];
                self.rdsec(block, &mut buf)?;

                for slot in 0..DPS {
                    let off = slot * DE;
                    let found = get32(&buf, off);
                    let len = get16(&buf, off + 6) as usize;

                    if found != 0
                        && len == want.len()
                        && &buf[off + 8..off + 8 + len] == want
                    {
                        put32(&mut buf, off, 0);
                        self.wrsec(block, &buf)?;
                        return Ok(());
                    }
                }
            }

            index += 1;
        }

        Err(())
    }

    // check for an empty dir
    fn empty(&self, dir: u32) -> Result<bool, ()> {
        let inode = self.rdino(dir)?;
        let mut index = 0;

        while (index * SEC as u32) < inode.size {
            let block = self.getblock(dir, index, false)?;

            if block != 0 {
                let mut buf = [0u8; SEC];
                self.rdsec(block, &mut buf)?;

                for slot in 0..DPS {
                    let off = slot * DE;
                    let found = get32(&buf, off);

                    if found == 0 {
                        continue;
                    }

                    let len = get16(&buf, off + 6) as usize;
                    let name = &buf[off + 8..off + 8 + len];

                    if name != b"." && name != b".." {
                        return Ok(false);
                    }
                }
            }

            index += 1;
        }

        Ok(true)
    }

    // set the .. entry
    fn dotdot(&self, dir: u32, parent: u32) -> Result<(), ()> {
        let inode = self.rdino(dir)?;
        let mut index = 0;

        while (index * SEC as u32) < inode.size {
            let block = self.getblock(dir, index, false)?;

            if block != 0 {
                let mut buf = [0u8; SEC];
                self.rdsec(block, &mut buf)?;

                for slot in 0..DPS {
                    let off = slot * DE;
                    let len = get16(&buf, off + 6) as usize;

                    if get32(&buf, off) != 0 && &buf[off + 8..off + 8 + len] == b".." {
                        put32(&mut buf, off, parent);
                        self.wrsec(block, &buf)?;
                        return Ok(());
                    }
                }
            }

            index += 1;
        }

        Err(())
    }

    // check for a child dir
    fn under(&self, top: u32, mut at: u32) -> Result<bool, ()> {
        for _ in 0..self.sup.inodes {
            if at == top {
                return Ok(true);
            }

            if at == self.sup.root {
                return Ok(false);
            }

            at = self.lookup(at, "..")?;
        }

        Ok(false)
    }

    // split a path
    fn parent<'a>(&self, path: &'a str) -> Result<(u32, &'a str), ()> {
        let path = path.trim_matches('/');

        let (dir, name) = match path.rfind('/') {
            Some(pos) => (&path[..pos], &path[pos + 1..]),
            None => ("", path),
        };

        if !okname(name) {
            return Err(());
        }

        let dir = if dir.is_empty() {
            self.sup.root
        } else {
            self.resolve(dir)?
        };

        if self.rdino(dir)?.mode != DIR {
            return Err(());
        }

        Ok((dir, name))
    }

    // find a path
    pub fn resolve(&self, path: &str) -> Result<u32, ()> {
        let mut ino = self.sup.root;

        for part in path.split('/') {
            if part.is_empty() || part == "." {
                continue;
            }

            if self.rdino(ino)?.mode != DIR {
                return Err(());
            }

            ino = self.lookup(ino, part)?;
        }

        Ok(ino)
    }

    // make a file
    pub fn create(&self, path: &str) -> Result<u32, ()> {
        let (dir, name) = self.parent(path)?;

        if self.findent(dir, name)?.is_some() {
            return Err(());
        }

        let ino = self.newino()?;
        let mut inode = Inode::new(FILE);
        inode.links = 1;

        if self.wrino(ino, &inode).is_err() {
            let _ = self.delino(ino);
            return Err(());
        }

        if self.addent(dir, name, ino, FILE).is_err() {
            let _ = self.delino(ino);
            return Err(());
        }

        Ok(ino)
    }

    // make a dir
    pub fn mkdir(&self, path: &str) -> Result<u32, ()> {
        let (dir, name) = self.parent(path)?;

        if self.findent(dir, name)?.is_some() {
            return Err(());
        }

        let ino = self.newino()?;

        let block = match self.newblock() {
            Ok(block) => block,
            Err(()) => {
                let _ = self.delino(ino);
                return Err(());
            }
        };

        let mut inode = Inode::new(DIR);
        inode.links = 2;
        inode.size = SEC as u32;
        inode.blocks = 1;
        inode.direct[0] = block;

        if self.wrino(ino, &inode).is_err() {
            let _ = self.delblock(block);
            let _ = self.delino(ino);
            return Err(());
        }

        let mut buf = [0u8; SEC];
        putent(&mut buf, 0, ino, DIR, ".");
        putent(&mut buf, 1, dir, DIR, "..");

        if self.wrsec(block, &buf).is_err() {
            let _ = self.delblock(block);
            let _ = self.delino(ino);
            return Err(());
        }

        if self.addent(dir, name, ino, DIR).is_err() {
            let _ = self.delblock(block);
            let _ = self.delino(ino);
            return Err(());
        }

        let mut parent = self.rdino(dir)?;
        parent.links += 1;
        self.wrino(dir, &parent)?;
        Ok(ino)
    }

    // remove a file
    pub fn unlink(&self, path: &str) -> Result<(), ()> {
        let (dir, name) = self.parent(path)?;
        let ino = self.lookup(dir, name)?;
        let mut inode = self.rdino(ino)?;

        if inode.mode != FILE {
            return Err(());
        }

        self.delent(dir, name)?;

        if inode.links > 1 {
            inode.links -= 1;
            self.wrino(ino, &inode)?;
        } else {
            self.trunc(ino)?;
            self.delino(ino)?;
        }

        Ok(())
    }

    // remove a dir
    pub fn rmdir(&self, path: &str) -> Result<(), ()> {
        let (dir, name) = self.parent(path)?;
        let ino = self.lookup(dir, name)?;

        if ino == self.sup.root || self.rdino(ino)?.mode != DIR || !self.empty(ino)? {
            return Err(());
        }

        self.delent(dir, name)?;
        self.trunc(ino)?;
        self.delino(ino)?;

        let mut parent = self.rdino(dir)?;
        if parent.links > 0 {
            parent.links -= 1;
            self.wrino(dir, &parent)?;
        }

        Ok(())
    }

    // rename a file
    pub fn rename(&self, from: &str, to: &str) -> Result<(), ()> {
        if from == to {
            return Ok(());
        }

        let (fromdir, fromname) = self.parent(from)?;
        let (todir, toname) = self.parent(to)?;
        let ino = self.lookup(fromdir, fromname)?;
        let inode = self.rdino(ino)?;

        if self.findent(todir, toname)?.is_some() {
            return Err(());
        }

        if inode.mode == DIR && self.under(ino, todir)? {
            return Err(());
        }

        self.addent(todir, toname, ino, inode.mode)?;
        self.delent(fromdir, fromname)?;

        if inode.mode == DIR && fromdir != todir {
            self.dotdot(ino, todir)?;

            let mut old = self.rdino(fromdir)?;
            if old.links > 0 {
                old.links -= 1;
                self.wrino(fromdir, &old)?;
            }

            let mut new = self.rdino(todir)?;
            new.links += 1;
            self.wrino(todir, &new)?;
        }

        Ok(())
    }

    // read file data
    pub fn read(&self, ino: u32, off: u32, buf: &mut [u8]) -> Result<usize, ()> {
        let inode = self.rdino(ino)?;

        if inode.mode != FILE {
            return Err(());
        }

        if off >= inode.size {
            return Ok(0);
        }

        let total = (inode.size - off) as usize;
        let count = total.min(buf.len());
        let mut done = 0;

        while done < count {
            let pos = off as usize + done;
            let index = (pos / SEC) as u32;
            let inner = pos % SEC;
            let len = (SEC - inner).min(count - done);
            let block = self.getblock(ino, index, false)?;

            if block == 0 {
                for byte in &mut buf[done..done + len] {
                    *byte = 0;
                }
            } else {
                let mut sector = [0u8; SEC];
                self.rdsec(block, &mut sector)?;
                buf[done..done + len].copy_from_slice(&sector[inner..inner + len]);
            }

            done += len;
        }

        Ok(done)
    }

    // write file data
    pub fn write(&self, ino: u32, off: u32, data: &[u8]) -> Result<usize, ()> {
        let inode = self.rdino(ino)?;

        if inode.mode != FILE {
            return Err(());
        }

        if off as u64 + data.len() as u64 > u32::MAX as u64 {
            return Err(());
        }

        let mut done = 0;

        while done < data.len() {
            let pos = off as usize + done;
            let index = (pos / SEC) as u32;
            let inner = pos % SEC;
            let len = (SEC - inner).min(data.len() - done);
            let block = self.getblock(ino, index, true)?;
            let mut sector = [0u8; SEC];

            if inner != 0 || len < SEC {
                self.rdsec(block, &mut sector)?;
            }

            sector[inner..inner + len].copy_from_slice(&data[done..done + len]);
            self.wrsec(block, &sector)?;
            done += len;
        }

        let end = off + done as u32;
        let mut inode = self.rdino(ino)?;

        if end > inode.size {
            inode.size = end;
            self.wrino(ino, &inode)?;
        }

        Ok(done)
    }

    // file size
    pub fn size(&self, ino: u32) -> Result<u32, ()> {
        Ok(self.rdino(ino)?.size)
    }

    // file type
    pub fn mode(&self, ino: u32) -> Result<u16, ()> {
        Ok(self.rdino(ino)?.mode)
    }

    // list a dir
    pub fn list(&self, path: &str, mut f: impl FnMut(&str, u32, u16)) -> Result<(), ()> {
        let ino = self.resolve(path)?;
        let inode = self.rdino(ino)?;

        if inode.mode != DIR {
            return Err(());
        }

        let mut index = 0;

        while (index * SEC as u32) < inode.size {
            let block = self.getblock(ino, index, false)?;

            if block != 0 {
                let mut buf = [0u8; SEC];
                self.rdsec(block, &mut buf)?;

                for slot in 0..DPS {
                    let off = slot * DE;
                    let found = get32(&buf, off);
                    let len = get16(&buf, off + 6) as usize;

                    if found != 0 && len <= NAMELEN {
                        if let Ok(name) = str::from_utf8(&buf[off + 8..off + 8 + len]) {
                            f(name, found, get16(&buf, off + 4));
                        }
                    }
                }
            }

            index += 1;
        }

        Ok(())
    }
}
