#!/bin/sh
# run.sh
# Build and run in QEMU with serial on stdio.

set -e

./build.sh
qemu-system-x86_64 -cdrom rynos.iso -m 256M -serial stdio -display none
