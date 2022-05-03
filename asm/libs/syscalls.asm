[bits 64]

; Symbol Exports
global syscall_write
global syscall_read
global syscall_open
global syscall_close
global syscall_sleep

%define SYSCALL_OPEN 0
%define SYSCALL_WRITE 1
%define SYSCALL_READ 2
%define SYSCALL_CLOSE 3

%define SYSCALL_SLEEP 4


section .text
syscall_write:
    mov rax, SYSCALL_WRITE
    int 0x80
    ret

syscall_read:
    mov rax, SYSCALL_READ
    int 0x80
    ret

syscall_open:
    mov rax, SYSCALL_OPEN
    int 0x80
    ret

syscall_close:
    mov rax, SYSCALL_CLOSE
    int 0x80
    ret

syscall_sleep:
    mov rax, SYSCALL_CLOSE
    int 0x80
    ret

