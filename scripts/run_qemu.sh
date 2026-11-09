#!/bin/sh
# run_qemu.sh
# Copyright (c) RynOS 2026
# Builds the ISO and boots it in QEMU: screen window, COM1 in terminal.

exec cargo xtask run "$@"
