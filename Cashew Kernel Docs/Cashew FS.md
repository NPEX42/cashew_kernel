## Design Goals
- Simplicity
- Speed
- CRUD Operations
- Device 'Files'



## Device Files
- 'Nix Style
	- /dev/tty*
	- /dev/mem
	- /sbd*
- Handled By Device File Drivers


### Architecture - V1.0
| Block | Name        | Description                                          |
|:-----:|:----------- |:---------------------------------------------------- |
|   1   | Superblock  | Identifies The Partition Type & Various Information. |
|   2   | Bitmap      |                                                      | 
| 16387 | Inode Table | Stores Inodes                                        |
|       |             |                                                      |
