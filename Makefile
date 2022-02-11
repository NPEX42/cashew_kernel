all: run


release: build
	cargo kimage --release

debug: build
	cargo kimage

run: build
	cargo krun --release


build:
	nasm test_elf.asm -felf64 -o test_elf.o
	ld test_elf.o test_elf
	cp test_elf ./initrd/bin/test_elf
	tar -cf initrd.img initrd
