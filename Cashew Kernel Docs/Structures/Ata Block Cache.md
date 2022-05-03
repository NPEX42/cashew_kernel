## The Problem
---
Disk I/O Is SLOW (16MB/s with PIO). When a block is read from disk once, we need to wait for the disk. Filesystem Operations will typically Operate on Continous stretches of Blocks. Read Operations are typically more Common
Than Writes. 


## The Solutions
---
- Keep a Copy of Block data in memory, when a Read Comes in for that block, return a copy of the in-memory block, skipping an Expensive Disk Operation. When a Write comes in for that block, we replace the data in the cache & then write the block to disk. Using this System, Blocks have to be read Once & Expensive Disk I/O Only Happens When Absolutely Necessary.


## Benchmarks
---
| Operation     | Raw (No-Cache) / Ticks           | Cache             | Difference |
| ------------- | -------------------------------- | ----------------- | ---------- |
| Read  (1x)    | 212 (212 / Block)                | 123 (123 / Block) | -89        | 
| Read  (10x)   | 3197 (319.7 / Block)             |                   |            |
| Read  (100x)  | 32937 (329.37 / Block)           |                   |            |
| ------------- | -------------------------------- | ----------------- |            |
| Write (1x)    | 239 (239 / Block)                |                   |            |
| Write (10x)   | 3268 (326.8 / Block)             |                   |            |
| Write (100x)  | 33265 (332.65 / Block)           |                   |            |
| ------------- | -------------------------------- | ----------------- |            |
| Average       | 12,115.33 ( 861.07 / Block)      |                   |            |
