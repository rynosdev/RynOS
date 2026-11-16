/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * build.rs
 * Build entry
 */

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // nasm object in the build dir
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let object = out.join("entry.o");

    // assemble the boot entry
    let status = Command::new("nasm")
        .args(["-f", "elf64", "src/boot/entry.asm", "-o"])
        .arg(&object)
        .status()
        .expect("nasm not found");

    if !status.success() {
        panic!("nasm failed");
    }

    // pass objects and linker script
    println!("cargo:rustc-link-arg={}", object.display());
    println!("cargo:rustc-link-arg=-Tlinker.ld");
    println!("cargo:rustc-link-arg=-no-pie");

    // rebuild when the entry or linker script changes
    println!("cargo:rerun-if-changed=src/boot/entry.asm");
    println!("cargo:rerun-if-changed=linker.ld");
}
