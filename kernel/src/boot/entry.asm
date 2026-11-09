; entry.asm
; Copyright (c) RynOS 2026
; Multiboot2 stub. GRUB enters in 32-bit protected mode, so we build page
; tables, switch to long mode and call kmain.

%define MB_MAGIC 0x36d76289

section .multiboot
align 8
header_start:
    dd 0xe85250d6                       ; multiboot2 magic
    dd 0                                ; i386 protected mode
    dd header_end - header_start
    dd -(0xe85250d6 + (header_end - header_start))
    ; end tag
    dw 0
    dw 0
    dd 8
header_end:

%macro VCHAR 1
    push eax
    push edi
    mov edi, 0xb8000
    add edi, [vpos]
    mov al, %1
    mov [edi], al
    mov byte [edi + 1], 0x0f
    add word [vpos], 2
    pop edi
    pop eax
%endmacro

%macro VCHAR64 1
    push rax
    push rdi
    mov rdi, 0xb8000
    movzx edi, word [vpos]
    add rdi, 0xb8000
    mov al, %1
    mov [rdi], al
    mov byte [rdi + 1], 0x0f
    add word [vpos], 2
    pop rdi
    pop rax
%endmacro

section .text
global _start
extern kmain

bits 32
_start:
    cmp eax, MB_MAGIC
    jne .die
    mov word [vpos], 0
    VCHAR 'A'
    mov esi, ebx                        ; keep multiboot info pointer
    VCHAR 'B'

    ; clear six page tables: pml4, pdpt and four pd pages
    mov edi, pml4
    xor eax, eax
    mov ecx, 6144
    rep stosd

    ; pml4[0] -> pdpt, pdpt[0..3] -> pd0..pd3
    mov eax, pdpt + 3
    mov [pml4], eax
    mov eax, pd0 + 3
    mov edi, pdpt
    mov ecx, 4
.fill_pdpt:
    mov [edi], eax
    add eax, 0x1000                     ; pd pages are consecutive
    add edi, 8
    loop .fill_pdpt

    ; pd entries: identity map all 4GB with 2MB pages
    mov edx, 0
    mov ebx, pd0
.fill:
    mov eax, edx
    shl eax, 21                         ; phys = 2MB * index
    or eax, 0x83                        ; present, write, page size
    mov [ebx + edx*8], eax
    mov dword [ebx + edx*8 + 4], 0
    inc edx
    cmp edx, 2048
    jne .fill

    mov eax, pml4
    mov cr3, eax
    VCHAR 'C'

    ; PAE
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; EFER.LME
    mov ecx, 0xc0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; paging on
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax
    VCHAR 'D'

    lgdt [gdt_ptr]
    VCHAR 'E'

    jmp 0x08:long_mode

.die:
    hlt
    jmp .die

bits 64
long_mode:
    VCHAR64 'F'
    mov esp, stack_top
    mov eax, 0x10
    mov ds, eax
    mov es, eax
    mov ss, eax

    mov edi, esi                        ; multiboot info pointer, zero extended
    VCHAR64 'K'
    call kmain

.halt:
    cli
    hlt
    jmp .halt

section .rodata
align 16
gdt64:
    dq 0                                ; null
    dq 0x00209a0000000000               ; code: present, ring 0, long
    dq 0x0000920000000000               ; data: present, ring 0, writable
gdt_end:
    dw gdt_end - gdt64 - 1             ; limit
    dq gdt64                            ; base (lgdt takes low 6 bytes)
gdt_ptr:
    dw gdt_end - gdt64 - 1
    dq gdt64

section .bss
vpos:
    resw 1
alignb 4096
pml4:
    resb 4096
pdpt:
    resb 4096
pd0:
    resb 4096
pd1:
    resb 4096
pd2:
    resb 4096
pd3:
    resb 4096
alignb 16
stack_bottom:
    resb 65536
stack_top:
