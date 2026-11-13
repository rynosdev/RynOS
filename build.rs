/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * build.rs
 * Assemble boot entry with NASM
 */

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let object = out.join("entry.o");

    let status = Command::new("nasm")
        .args(["-f", "elf64", "src/boot/entry.asm", "-o"])
        .arg(&object)
        .status()
        .expect("nasm not found");

    if !status.success() {
        panic!("nasm failed");
    }

    println!("cargo:rustc-link-arg={}", object.display());
    println!("cargo:rerun-if-changed=src/boot/entry.asm");
}
