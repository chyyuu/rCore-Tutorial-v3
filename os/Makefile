# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# BOARD
BOARD := qemu
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# KERNEL ENTRY (S-Mode kernel will be loaded here)
KERNEL_ENTRY_PA := 0x80200000

# M-Mode entry (for -bios none)
M_MODE_ENTRY_PA := 0x80000000

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# Disassembly
DISASM ?= -x

# Default build (with rustsbi)
build: env $(KERNEL_BIN)

# Build for -bios none (with M-Mode support)
build-nobios: env kernel-nobios
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $(KERNEL_BIN)

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

# Default kernel build (S-Mode only, uses rustsbi)
kernel:
	@echo Platform: $(BOARD) with RustSBI
	@cp src/linker-$(BOARD).ld src/linker.ld
	@cargo build $(MODE_ARG)
	@rm src/linker.ld

# Kernel build with M-Mode support (for -bios none)
kernel-nobios:
	@echo "Platform: $(BOARD) with M-Mode SBI - no external bootloader"
	@cp src/linker-$(BOARD)-nobios.ld src/linker.ld
	@cargo build $(MODE_ARG) --features nobios
	@rm src/linker.ld

clean:
	@cargo clean

disasm: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less

disasm-vim: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	@vim $(DISASM_TMP)
	@rm $(DISASM_TMP)

run: run-inner

# Original QEMU args with rustsbi bootloader
QEMU_ARGS := -machine virt \
			 -nographic \
			 -bios $(BOOTLOADER) \
			 -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)

# New QEMU args with -bios none (uses our M-Mode implementation)
# Binary is loaded at M-Mode entry (0x80000000)
QEMU_ARGS_NOBIOS := -machine virt \
			 -nographic \
			 -bios none \
			 -device loader,file=$(KERNEL_BIN),addr=$(M_MODE_ENTRY_PA)

QEMU_NAME := qemu-system-riscv64
qemu-version-check:
	@sh scripts/qemu-ver-check.sh $(QEMU_NAME)

run-inner: qemu-version-check build
	@qemu-system-riscv64 $(QEMU_ARGS)

# Run with -bios none (M-Mode implementation)
run-nobios: qemu-version-check build-nobios
	@qemu-system-riscv64 $(QEMU_ARGS_NOBIOS)

debug: qemu-version-check build
	@tmux new-session -d \
		"qemu-system-riscv64 $(QEMU_ARGS) -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

debug-nobios: qemu-version-check build-nobios
	@tmux new-session -d \
		"qemu-system-riscv64 $(QEMU_ARGS_NOBIOS) -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

gdbserver: qemu-version-check build
	@qemu-system-riscv64 $(QEMU_ARGS) -s -S

gdbserver-nobios: qemu-version-check build-nobios
	@qemu-system-riscv64 $(QEMU_ARGS_NOBIOS) -s -S

gdbclient:
	@riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'

.PHONY: build build-nobios env kernel kernel-nobios clean disasm disasm-vim run-inner run-nobios gdbserver gdbserver-nobios gdbclient qemu-version-check debug-nobios
