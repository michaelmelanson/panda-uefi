.PHONY: all clean
all: loader kernel startup
clean:
	cargo clean
	rm -Rf build

run: all
	/Users/michael/dev/qemu/build/qemu-system-x86_64 \
	  -nodefaults \
      -vga virtio \
	  -machine q35,accel=kvm:tcg \
	  -m 1024M \
	  -drive if=pflash,unit=0,format=raw,readonly=on,file=vendor/OVMF_CODE.fd \
	  -drive if=pflash,unit=1,format=raw,file=vendor/OVMF_VARS-1024x768.fd \
	  -drive format=raw,file=fat:rw:build \
	  -monitor vc:1024x768 \
	  -nic user,model=virtio-net-pci \
      -d trace:ahci_*,trace:ide_*,trace:cmd_identify,int \
	  -serial stdio \
	  -D qemu.log

.PHONY: loader
loader:
	cd panda-loader && cargo +nightly build --release
	mkdir -p build/EFI/BOOT
	cp target/x86_64-unknown-uefi/release/panda-loader.efi build/EFI/BOOT/BootX64.efi

.PHONY: kernel
kernel:
	cd panda-kernel && cargo +nightly build --release 
	mkdir -p build/EFI
	cp target/x86_64-panda-elf/release/panda-kernel build/EFI/kernel.elf

.PHONY: startup
startup:
	mkdir -p build/
	echo "\EFI\BOOT\BOOTX64.EFI" > build/startup.nsh