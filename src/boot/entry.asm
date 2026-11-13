; /*
;  * License: GPL-2.0-only
;  * Copyright (C) 2026, rynosdev
;  * entry.asm
;  * Kernel entry
;  */

BITS 32

MB2_MAGIC equ 0xE85250D6

section .multiboot
align 8
    dd MB2_MAGIC
    dd 0
    dd header_end - header_start
    dd -(MB2_MAGIC + 0 + (header_end - header_start))
header_start:
    dw 0
    dw 0
    dd 8
header_end:

section .text
global _start
extern kmain

_start:
    cli
    mov esp, stack_top

    ; zero page tables
    mov edi, pml4
    xor eax, eax
    mov ecx, (pd_end - pml4) / 4
    rep stosd

    ; pml4[0] -> pdpt
    mov eax, pdpt
    or eax, 3
    mov [pml4], eax

    ; pdpt[0] -> pd
    mov eax, pd
    or eax, 3
    mov [pdpt], eax

    ; identity map first 1 GiB with 2 MiB pages
    mov eax, 0x83
    mov ecx, 512
    mov edi, pd
.map:
    mov [edi], eax
    mov dword [edi + 4], 0
    add eax, 0x200000
    add edi, 8
    loop .map

    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    mov eax, pml4
    mov cr3, eax

    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    mov eax, cr0
    or eax, 0x80000001
    mov cr0, eax

    lgdt [gdt_ptr]
    jmp 0x08:long_mode

BITS 64

long_mode:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    mov rsp, stack_top
    xor rbp, rbp
    call kmain

.halt:
    hlt
    jmp .halt

section .rodata
align 8
gdt:
    dq 0
    dq 0x00AF9A000000FFFF
    dq 0x00CF92000000FFFF
gdt_end:
gdt_ptr:
    dw gdt_end - gdt - 1
    dd gdt

section .bss
align 4096
pml4:
    resb 4096
pdpt:
    resb 4096
pd:
    resb 4096
pd_end:

align 16
stack_bottom:
    resb 16384
stack_top:
