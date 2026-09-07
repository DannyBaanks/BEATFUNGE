bits 16
org 0x7c00

ENGINE_SECTORS equ 4

start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7c00
    sti

    mov [0x0500], dl
    mov bx, 0x0100
    mov ah, 0x02
    mov al, ENGINE_SECTORS
    mov ch, 0
    mov cl, 2
    mov dh, 0
    int 0x13
    jc disk_error
    jmp 0x0000:0x0100

disk_error:
    cli
.halt:
    hlt
    jmp .halt

times 510 - ($ - $$) db 0
dw 0xaa55
