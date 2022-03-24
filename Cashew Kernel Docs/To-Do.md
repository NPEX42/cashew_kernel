# Cashew Kernel - To-Do
---

## Urgent

- [x] Physical Memory Manager


## Important
- [ ] Virtual Filesystem
	- [ ] Pipes
	- [ ] Directories
	- [ ] Files
	- [ ] Device Formatting
	- [ ] CashewFS 
- [ ] ACPI I/O 
	- [ ] ACPI Restart
	- [x] ACPI Shutdown - https://wiki.osdev.org/ACPI
- [ ] PCI Device I/O 
	- [ ] PCI Bus Enumeration
	- [ ] Network Drivers 
		- [ ] RTL8139 Network Driver
		- [ ] AMD PCNET Network Driver
		- [ ] Network Stack
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
		- [ ] FIFO Pipe
		- [ ] Null device
		- [ ] PRNG
	- [ ] Block Devices
		- [ ] ATA
			- [x] PI/O Mode
			- [ ] DMA Mode
		- [ ] Memory Disk 

## Refactoring
 - [ ] Proper API Module - cashew::api
	 - [ ] 


## Backburner
- [ ] RTC Century Register Support
- [ ] Async/Await Support
	- [ ] PIT Waker Support
	- [ ] Round-Robin Scheduler
