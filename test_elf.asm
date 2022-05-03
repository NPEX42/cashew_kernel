[bits 64]

global _start



_start:
                            ; sleep(5.0f)
  mov rax, 9                ; syscall number for SLEEP
  mov rdi, __float64__(5.0) ; time to sleep in seconds
  int 0x80
                            ; exit(0)
  mov rax, 1                ; syscall number for EXIT
  mov rdi, 0                ; no error
  int 0x80