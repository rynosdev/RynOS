; License: GPL-2.0-only
; Copyright (C) 2026, rynosdev
; entry.asm
; Kernel entry

BITS 32

MAGIC equ 0xE85250D6

; multiboot2 header
section .multiboot
align 8
    dd MAGIC
    dd 0
    dd hend - hstart
    dd -(MAGIC + 0 + (hend - hstart))
hstart:
    ; request a framebuffer
    dw 5
    dw 0
    dd 20
    dd 800
    dd 600
    dd 32
    dd 0
    dw 0
    dw 0
    dd 8
hend:

; 32 bit entry
section .text
global start
extern kmain

start:
    cli
    mov esp, stop
    mov esi, eax

    ; clear the page tables
    mov edi, pml4
    xor eax, eax
    mov ecx, (pdend - pml4) / 4
    rep stosd

    ; pml4 -> pdpt -> four page dirs
    mov eax, pdpt
    or eax, 3
    mov [pml4], eax

    ; map 4 GiB with 2 MiB pages
    mov edi, pdpt
    mov eax, pd
    mov ecx, 4
.pdpt:
    or eax, 3
    mov [edi], eax
    add eax, 4096
    add edi, 8
    loop .pdpt

    mov edi, pd
    mov eax, 0x83
    mov ecx, 2048
.map:
    mov [edi], eax
    mov dword [edi + 4], 0
    add eax, 0x200000
    add edi, 8
    loop .map

; turn on long mode
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

    lgdt [gdtp]
    jmp 0x08:lmode

BITS 64

; 64 bit mode
lmode:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    mov rsp, stop
    xor rbp, rbp
    mov rdi, rsi
    mov rsi, rbx
    mov esi, esi
    call kmain

.halt:
    hlt
    jmp .halt

; gdt for long mode
section .rodata
align 8
gdt:
    dq 0
    dq 0x00AF9A000000FFFF
    dq 0x00CF92000000FFFF
gdtend:
gdtp:
    dw gdtend - gdt - 1
    dd gdt

; page tables and stack
section .bss
align 4096
pml4:
    resb 4096
pdpt:
    resb 4096
pd:
    resb 4096 * 4
pdend:

align 16
sbot:
    resb 16384
stop:
