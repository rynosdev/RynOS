# linker.ld
# Kernel memory layout. Loaded at 1 MiB by GRUB.

ENTRY(_start)

SECTIONS
{
    . = 1M;

    .text : ALIGN(4K)
    {
        KEEP(*(.multiboot))
        *(.text .text.*)
    }

    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4K)
    {
        *(.data .data.*)
    }

    .bss : ALIGN(4K)
    {
        *(.bss .bss.*)
        *(COMMON)
    }
}
