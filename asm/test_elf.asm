[bits 64]

global _start
%include "asm/libs/syscalls.i"
extern syscall_sleep

SECTION .text
_start:
                            ; sleep(5.0f)
  mov rdi, __float64__(5.0) ; time to sleep in seconds
  call syscall_sleep
                            ; exit(0)
  mov rax, 1                ; syscall number for EXIT
  mov rdi, 0                ; no error
  int 0x80