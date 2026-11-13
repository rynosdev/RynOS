#!/bin/sh
# build.sh
# Build the kernel and the iso image.

set -e

cargo build --release

mkdir -p iso/boot/grub
cp target/x86_64-unknown-none/release/rynos iso/boot/kernel.bin
cp grub.cfg iso/boot/grub/grub.cfg
grub-mkrescue -o rynos.iso iso >/dev/null 2>&1

echo "iso: rynos.iso"
