# Cashew Kernel - To-Do
---

## Urgent

- [x] Physical Memory Manager


## Important
- [ ] Virtual Filesystem
	- [ ] Pipes
	- [ ] Directories
	- [x] Files
	- [x] Device Formatting
	- [ ] CashewFS 
- [ ] ACPI I/O 
	- [ ] ACPI Restart
	- [x] ACPI Shutdown - https://wiki.osdev.org/ACPI
- [ ] PCI Device I/O 
	- [x] PCI Bus Enumeration
	- [ ] Network Drivers 
		- [x] RTL8139 Network Driver
		- [ ] AMD PCNET Network Driver
		- [x] Network Stack
		- [ ] DHCP Client
		- [ ] Ping Utility
		- [ ] Sockets
- [ ] Userspace
	- [ ] Support for ELF64 Programs
		- [ ] Section Loading
		- [ ] Switching To Usermode
		- [ ] POSIX System Calls
		- [ ] libc Implementation
- [ ] I/O Devices
	- [ ]  Char device Abstractions
		- [x] Terminal
		- [x] Serial  
		- [x] FIFO Pipe
		- [x] Null device
		- [ ] PRNG
	- [ ] Block Devices
		- [ ] ATA
			- [x] PI/O Mode
			- [ ] DMA Mode
		- [x] Memory Disk 

## Refactoring
 - [ ] Proper API Module - cashew::api
	 - [ ] 


## Backburner
- [ ] RTC Century Register Support
- [ ] Async/Await Support
	- [ ] PIT Waker Support
	- [ ] Round-Robin Scheduler



## Quality Of Life
- [ ]  Full ANSI Control Code Support - Issue [#2](https://github.com/NPEX42/cashew_kernel/issues/2)
	- [ ] Color
	- [ ] Position
		- [ ] Save
		- [ ] Load
		- [ ] Home
		- [ ] Set
	- [ ] Clear
- [ ] Clean Up External API - Issue [#3](https://github.com/NPEX42/cashew_kernel/issues/3)
- [ ] Improved Command Parser - Issue [#4](https://github.com/NPEX42/cashew_kernel/issues/4)
