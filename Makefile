ARCH ?= x86_64
TARGET ?= $(ARCH)-rcpu_os

KERNEL := build/kernel-$(ARCH).bin
ISO := build/rcpu_os-$(ARCH).iso
RUST_OS := target/$(TARGET)/debug/librcpu_os.a

LINKER_SCRIPT := src/arch/$(ARCH)/linker.ld
MK_GRUB_CFG := src/arch/$(ARCH)/mkgrubcfg.sh
ASM_SRC_FILES := $(wildcard src/arch/$(ARCH)/*.asm)
ASM_OBJ_FILES := $(patsubst src/arch/$(ARCH)/%.asm, \
	build/arch/$(ARCH)/%.o, $(ASM_SRC_FILES))


.PHONY: all clean run iso kernel

all: $(KERNEL)

clean:
	rm -r build target

run: $(ISO)
	qemu-system-x86_64 -cdrom $(ISO)

iso: $(ISO)

$(ISO): $(KERNEL) $(GRUB_CFG)
	mkdir -p build/isofiles/boot/grub
	cp modules/* build/isofiles/boot
	cp $(KERNEL) build/isofiles/boot/kernel.bin
	$(MK_GRUB_CFG) modules > build/isofiles/boot/grub/grub.cfg
	grub-mkrescue -o $(ISO) build/isofiles
	rm -r build/isofiles

$(KERNEL): kernel $(RUST_OS) $(ASM_OBJ_FILES) $(LINKER_SCRIPT)
	ld -n -T $(LINKER_SCRIPT) -o $(KERNEL) \
		$(ASM_OBJ_FILES) $(RUST_OS)

kernel:
	RUST_TARGET_PATH=$(shell pwd) cargo build --target $(TARGET)

# compile assembly files
build/arch/$(ARCH)/%.o: src/arch/$(ARCH)/%.asm
	mkdir -p $(shell dirname $@)
	nasm -felf64 $< -o $@
