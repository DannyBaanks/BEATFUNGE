bits 16
org 0x0100

GRID           equ 0x0a00
STATE          equ 0x0900
TAPE           equ 0x0c00
TAPE_END       equ 0x1400
BOOT_DRIVE     equ 0x0500
GRID_SECTOR    equ 6

IP_X           equ STATE + 0
IP_Y           equ STATE + 1
IP_DIR         equ STATE + 2
REG            equ STATE + 4
TEMPO          equ STATE + 6
DURATION       equ STATE + 8
TAPE_PTR       equ STATE + 10

RIGHT          equ 0
LEFT           equ 1
UP             equ 2
DOWN           equ 3

start:
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov [IP_X], al
    mov [IP_Y], al
    mov [IP_DIR], al
    mov word [REG], 0
    mov word [TEMPO], 140
    mov word [DURATION], 9
    mov word [TAPE_PTR], TAPE

    ; The boot sector reserved sectors 2..5 for this engine; sector 6 is grid.
    mov bx, GRID
    mov ah, 0x02
    mov al, 1
    mov ch, 0
    mov cl, GRID_SECTOR
    mov dh, 0
    mov dl, [BOOT_DRIVE]
    int 0x13
    jc halt

    mov di, TAPE
    mov cx, (TAPE_END - TAPE) / 2
    xor ax, ax
    rep stosw

main_loop:
    call fetch
    cmp al, '>'
    je dir_right
    cmp al, '<'
    je dir_left
    cmp al, '^'
    je dir_up
    cmp al, 'v'
    je dir_down
    cmp al, '_'
    je branch_horizontal
    cmp al, '|'
    je branch_vertical
    cmp al, '#'
    je bridge
    cmp al, 'T'
    je set_tempo
    cmp al, 'D'
    je set_duration
    cmp al, 'N'
    je set_note
    cmp al, '.'
    je play_note
    cmp al, 'P'
    je pause
    cmp al, 'H'
    je halt
    cmp al, '@'
    je halt
    cmp al, '+'
    je add_one
    cmp al, '-'
    je sub_one
    cmp al, '{'
    je tape_left
    cmp al, '}'
    je tape_right
    cmp al, 'R'
    je tape_read
    cmp al, 'W'
    je tape_write
    cmp al, 'S'
    je speaker_toggle
    call advance
    jmp main_loop

dir_right:
    mov byte [IP_DIR], RIGHT
    jmp step
dir_left:
    mov byte [IP_DIR], LEFT
    jmp step
dir_up:
    mov byte [IP_DIR], UP
    jmp step
dir_down:
    mov byte [IP_DIR], DOWN
    jmp step

branch_horizontal:
    cmp word [REG], 0
    jne dir_left
    jmp dir_right
branch_vertical:
    cmp word [REG], 0
    jne dir_up
    jmp dir_down
bridge:
    call advance
    jmp step

set_tempo:
    call advance
    call fetch
    sub al, '0'
    cmp al, 9
    ja step
    xor ah, ah
    mov bx, ax
    shl bx, 1
    mov ax, [tempo_table + bx]
    mov [TEMPO], ax
    jmp step

set_duration:
    call advance
    call fetch
    sub al, '0'
    cmp al, 9
    ja step
    xor ah, ah
    mov [DURATION], ax
    jmp step

set_note:
    xor di, di
.digit:
    call advance
    call fetch
    cmp al, '0'
    jb .done
    cmp al, '9'
    ja .done
    sub al, '0'
    xor ah, ah
    mov bx, di
    shl di, 1
    shl bx, 1
    shl bx, 1
    shl bx, 1
    add di, bx
    add di, ax
    jmp .digit
.done:
    mov [REG], di
    jmp main_loop

play_note:
    call speaker_on_note
    call wait_duration
    call speaker_off
    jmp step
pause:
    call speaker_off
    call wait_duration
    jmp step

add_one:
    inc word [REG]
    jmp step
sub_one:
    dec word [REG]
    jmp step

tape_left:
    mov bx, [TAPE_PTR]
    dec bx
    cmp bx, TAPE
    jae .store
    mov bx, TAPE_END - 1
.store:
    mov [TAPE_PTR], bx
    jmp step
tape_right:
    mov bx, [TAPE_PTR]
    inc bx
    cmp bx, TAPE_END
    jb .store
    mov bx, TAPE
.store:
    mov [TAPE_PTR], bx
    jmp step
tape_read:
    mov bx, [TAPE_PTR]
    xor ax, ax
    mov al, [bx]
    mov [REG], ax
    jmp step
tape_write:
    mov bx, [TAPE_PTR]
    mov ax, [REG]
    mov [bx], al
    jmp step

speaker_toggle:
    in al, 0x61
    xor al, 3
    out 0x61, al
    jmp step

step:
    call advance
    jmp main_loop

; AL = grid[y * 32 + x]. Fetch never changes the instruction pointer.
fetch:
    xor ax, ax
    mov al, [IP_Y]
    shl ax, 1
    shl ax, 1
    shl ax, 1
    shl ax, 1
    shl ax, 1
    mov bx, ax
    xor ax, ax
    mov al, [IP_X]
    add bx, ax
    add bx, GRID
    mov al, [bx]
    ret

; Move exactly one cell in the current direction, with a 32 x 16 torus.
advance:
    mov al, [IP_DIR]
    cmp al, RIGHT
    je .right
    cmp al, LEFT
    je .left
    cmp al, UP
    je .up
.down:
    inc byte [IP_Y]
    cmp byte [IP_Y], 16
    jb .done
    mov byte [IP_Y], 0
    ret
.right:
    inc byte [IP_X]
    cmp byte [IP_X], 32
    jb .done
    mov byte [IP_X], 0
    ret
.left:
    dec byte [IP_X]
    cmp byte [IP_X], 0xff
    jne .done
    mov byte [IP_X], 31
    ret
.up:
    dec byte [IP_Y]
    cmp byte [IP_Y], 0xff
    jne .done
    mov byte [IP_Y], 15
.done:
    ret

speaker_on_note:
    mov bx, [REG]
    cmp bx, 19
    jb speaker_off
    mov dx, 0x0012
    mov ax, 0x34dc
    div bx
    mov bx, ax
    mov al, 0xb6
    out 0x43, al
    mov al, bl
    out 0x42, al
    mov al, bh
    out 0x42, al
    in al, 0x61
    or al, 3
    out 0x61, al
    ret

speaker_off:
    in al, 0x61
    and al, 0xfc
    out 0x61, al
    ret

; Deliberately uses CPU work, not INT 1Ah. The BEAT QEMU test exposed that
; BIOS tick polling can collapse note durations in this capture configuration.
wait_duration:
    push cx
    push dx
    mov dx, [DURATION]
    inc dx
    shl dx, 10
.outer:
    mov cx, 0xffff
.inner:
    loop .inner
    dec dx
    jnz .outer
    pop dx
    pop cx
    ret

halt:
    call speaker_off
    ; Let QEMU's WAV backend drain its PC-speaker queue before debug-exit.
    ; On physical hardware this is simply a short silent tail before halt.
    mov cx, 6
.drain:
    call wait_duration
    loop .drain
    ; QEMU's optional isa-debug-exit observes this; real hardware ignores it.
    xor al, al
    out 0xf4, al
    cli
.loop:
    hlt
    jmp .loop

tempo_table:
    dw 60, 80, 100, 120, 140, 160, 180, 200, 220, 240

times 2048 - ($ - $$) db 0
