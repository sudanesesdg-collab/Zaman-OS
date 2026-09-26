[BITS 16]
[ORG 0x7C00]

start:
    mov dx, 0x3F8
    mov al, 13
    out dx, al
    mov al, 10
    out dx, al
    mov al, 'Z'
    out dx, al
    mov al, 'A'
    out dx, al
    mov al, 'M'
    out dx, al
    mov al, 'A'
    out dx, al
    mov al, 'N'
    out dx, al
    mov al, ' '
    out dx, al
    mov al, 'O'
    out dx, al
    mov al, 'S'
    out dx, al
    mov al, 13
    out dx, al
    mov al, 10
    out dx, al
    mov al, 'H'
    out dx, al
    mov al, 'e'
    out dx, al
    mov al, 'l'
    out dx, al
    mov al, 'l'
    out dx, al
    mov al, 'o'
    out dx, al
    mov al, '!'
    out dx, al
    mov al, 13
    out dx, al
    mov al, 10
    out dx, al

hang:
    cli
    hlt
    jmp hang

times 510 - ($ - $$) db 0
dw 0xAA55
