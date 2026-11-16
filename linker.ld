/*
 * License: GPL-2.0-only
 * Copyright (C) 2026, rynosdev
 * linker.ld
 * Linker script
 */

ENTRY(start)

/* kernel starts at 1 MiB */
SECTIONS
{
    . = 1M;

    /* code and multiboot header */
    .text : ALIGN(4K)
    {
        KEEP(*(.multiboot))
        *(.text .text.*)
    }

    /* read only data */
    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.*)
    }

    /* data */
    .data : ALIGN(4K)
    {
        *(.data .data.*)
    }

    /* zeroed data */
    .bss : ALIGN(4K)
    {
        *(.bss .bss.*)
        *(COMMON)
    }
}
