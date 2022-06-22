## Instructions
| Opcode | ID  | Notes                                      |
| ------ | --- | ------------------------------------------ |
| PUSHI  | 000 | push(imm)                                  |
| POP    | 001 | pop();                                     |
| ADD    | 002 | a = pop(); b = pop(); push(a + b)          |
| SUB    | 003 | a = pop(); b = pop(); push(a - b)          |
| DIV    | 004 | a = pop(); b = pop(); push(a / b)          |
| MUL    | 005 | a = pop(); b = pop(); push(a * b)          |
| GT     | 006 | a = pop(); b = pop(); push(a > b ? -1 : 0) |
| LT     | 006 | a = pop(); b = pop(); push(a < b ? -1 : 0) | 
