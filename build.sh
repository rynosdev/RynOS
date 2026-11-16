#!/bin/sh
# License: GPL-2.0-only
# Copyright (C) 2026, rynosdev
# build.sh
# Build script

set -e

# build the kernel
cargo build --release --target x86_64-unknown-none

# copy files into the iso tree
mkdir -p iso/boot/grub
cp target/x86_64-unknown-none/release/rynos iso/boot/kernel.bin
cp grub.cfg iso/boot/grub/grub.cfg
# make the iso
grub-mkrescue -o rynos.iso iso >/dev/null 2>&1

echo "iso: rynos.iso"
