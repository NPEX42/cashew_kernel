all: build


release: build
	cargo kimage --release

debug: build
	cargo kimage

run: build
	cargo krun --release


build:
	tar -cf initrd.img initrd
	truncate -s +524288 initrd.img
