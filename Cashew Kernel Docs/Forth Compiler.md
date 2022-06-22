## Overview

## Grammar

```bnf
Program    := (Statement)+ <EOI>
Statement  := Command | WordDecl
Command    := Number | String | Word | Branch
Number     := <Int>
WordDecl   := ':' Identifier Comment? Command+ ';'  
Word       := Identifier | '.'
Identifier := [A-Za-z_@!?$][A-Za-z0-9_@!?$]+
Comment    := '(' .* ')'
```


----
## Compiler Modes

Native Compiler:
- Compiles directly to x64 Machine Code
- Fastest

 Virtual Machine:
 - Compiles To [FVM](FVM) Bytecode 
 - Portable
 
Interpreted:
- Slowest
- Easiest to Debug / Develop

