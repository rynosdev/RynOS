#!/bin/sh
# License: GPL-2.0-only
# Copyright (C) 2026, rynosdev
# run.sh
# Run script

set -e

# build first
./build.sh

# make a disk image once
if [ ! -f disk.img ]; then
    truncate -s 64M disk.img
fi

# start qemu
qemu-system-x86_64 -cdrom rynos.iso -drive file=disk.img,format=raw,if=ide \
    -m 256M -boot order=d -vga std
