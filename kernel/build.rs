/*
 * build.rs
 * Copyright (c) RynOS 2026
 * Runs nasm on the boot stub and links the object in.
 */

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let obj = out_dir.join("entry.o");

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    println!("cargo:rerun-if-changed=src/boot/entry.asm");
    println!("cargo:rerun-if-changed=linker.ld");
    println!(
        "cargo:rustc-link-arg=-T{}",
        manifest.join("linker.ld").display()
    );

    let status = Command::new("nasm")
        .args(["-f", "elf64", "src/boot/entry.asm", "-o"])
        .arg(&obj)
        .status()
        .expect("nasm failed to run");
    if !status.success() {
        panic!("nasm failed");
    }

    println!("cargo:rustc-link-arg={}", obj.display());
}
