/*
 * main.rs
 * Copyright (c) RynOS 2026
 * Build/run/test/image commands. Entry: cargo xtask <command>.
 */

use std::path::{Path, PathBuf};
use std::process::Command;

const QEMU_TIMEOUT_SECS: u64 = 15;
const MARKER: &str = "RynOS v0.0.1";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn sh(cmd: &mut Command) {
    let status = cmd.status().expect("command failed to start");
    if !status.success() {
        std::process::exit(1);
    }
}

// the kernel dir has its own .cargo config with the target set
fn cargo_build() {
    let dir = root().join("kernel");
    sh(Command::new("cargo").arg("build").current_dir(&dir));
}

fn kernel_bin() -> PathBuf {
    root().join("target/x86_64-unknown-none/debug/rynkernel")
}

fn build_image() -> PathBuf {
    cargo_build();
    let r = root();
    std::fs::copy(kernel_bin(), r.join("iso/boot/kernel.bin")).expect("copy kernel");
    // grub needs the kernel as a plain file next to grub.cfg
    let iso = r.join("rynos.iso");
    sh(
        Command::new("grub-mkrescue")
            .arg("-o")
            .arg(&iso)
            .arg(r.join("iso"))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null()),
    );
    iso
}

fn run(iso: &Path) -> ! {
    sh(
        Command::new("qemu-system-x86_64")
            .args(["-m", "128", "-cdrom"])
            .arg(iso)
            .args(["-serial", "stdio", "-no-reboot", "-no-shutdown"]),
    );
    std::process::exit(0);
}

// headless boot test: watch serial for the boot marker
fn test(iso: &Path) -> ! {
    let log = root().join("target/test-serial.log");
    let mut qemu = Command::new("qemu-system-x86_64")
        .args(["-m", "128", "-cdrom"])
        .arg(iso)
        .args(["-display", "none", "-no-reboot"])
        .arg("-serial")
        .arg(format!("file:{}", log.display()))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("qemu failed");
    std::thread::sleep(std::time::Duration::from_secs(QEMU_TIMEOUT_SECS));
    let _ = qemu.kill();
    let _ = qemu.wait();
    let out = std::fs::read_to_string(&log).unwrap_or_default();
    if out.contains(MARKER) {
        println!("test ok: {} in serial", MARKER);
        std::process::exit(0);
    } else {
        eprintln!("test failed: no {} in serial", MARKER);
        eprintln!("--- serial log ---");
        eprintln!("{}", out);
        std::process::exit(1);
    }
}

fn main() {
    let cmd = std::env::args().nth(1).unwrap_or_default();
    let iso = match cmd.as_str() {
        "build" => {
            cargo_build();
            std::process::exit(0);
        }
        "image" => build_image(),
        "run" => run(&build_image()),
        "test" => test(&build_image()),
        other => {
            eprintln!("usage: cargo xtask build|run|test|image (got '{}')", other);
            std::process::exit(2);
        }
    };
    let _ = iso;
}
