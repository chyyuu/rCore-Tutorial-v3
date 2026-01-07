# Architecture selection: riscv32 or riscv64 (default)
ARCH ?= riscv64

# QEMU path (can be overridden)
# For custom QEMU builds, set QEMU_DIR
# QEMU_DIR ?= /path/to/custom/qemu/build

# Building configuration based on architecture
ifeq ($(ARCH), riscv32)
    TARGET := riscv32imac-unknown-none-elf
    QEMU_NAME := qemu-system-riscv32
    OBJDUMP := rust-objdump --arch-name=riscv32
    OBJCOPY := rust-objcopy --binary-architecture=riscv32
    GDB_ARCH := riscv:rv32
else
    TARGET := riscv64gc-unknown-none-elf
    QEMU_NAME := qemu-system-riscv64
    OBJDUMP := rust-objdump --arch-name=riscv64
    OBJCOPY := rust-objcopy --binary-architecture=riscv64
    GDB_ARCH := riscv:rv64
endif

# Determine QEMU path
ifdef QEMU_DIR
    QEMU_PATH := $(QEMU_DIR)/$(QEMU_NAME)
else
    # Try to find QEMU in PATH first, then check common custom build locations
    QEMU_PATH := $(shell which $(QEMU_NAME) 2>/dev/null)
    ifeq ($(QEMU_PATH),)
        # Check for custom QEMU build location
        QEMU_PATH := $(wildcard /home/*/thecodes/qemu-*/build/$(ARCH)-softmmu/$(QEMU_NAME))
    endif
endif

# Show which QEMU will be used
$(info $(QEMU_PATH))

MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm
FS_IMG := ../user/target/$(TARGET)/$(MODE)/fs.img
APPS := ../user/src/bin/*

# BOARD
BOARD := qemu
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# KERNEL ENTRY (S-Mode kernel will be loaded here)
KERNEL_ENTRY_PA := 0x80200000

# M-Mode entry (for -bios none)
M_MODE_ENTRY_PA := 0x80000000

# Disassembly
DISASM ?= -x

# Run usertests or usershell
TEST ?=

# Default build (with rustsbi) - only for riscv64
build: env $(KERNEL_BIN) fs-img

# Build for -bios none (with M-Mode support)
build-nobios: env kernel-nobios fs-img-nobios
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $(KERNEL_BIN)

env:
	(rustup target list | grep "$(TARGET) (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

fs-img: $(APPS)
	@cd ../user && make build TEST=$(TEST)
	@rm -f $(FS_IMG)
	@cd ../easy-fs-fuse && cargo run --release -- -s ../user/src/bin/ -t ../user/target/riscv64gc-unknown-none-elf/release/

fs-img-nobios: $(APPS)
	@cd ../user && make build ARCH=$(ARCH) TEST=$(TEST)
	@rm -f $(FS_IMG)
	@cd ../easy-fs-fuse && cargo run --release -- -s ../user/src/bin/ -t ../user/target/$(TARGET)/release/

$(APPS):

# Default kernel build (S-Mode only, uses rustsbi) - only for riscv64
kernel:
ifeq ($(ARCH), riscv32)
	$(error RustSBI mode is only supported for riscv64. Use 'make run-nobios ARCH=riscv32' instead.)
endif
	@cd ../user && make build TEST=$(TEST)
	@echo Platform: $(BOARD) with RustSBI [$(ARCH)]
	@cp src/linker-$(BOARD).ld src/linker.ld
	@cargo build $(MODE_ARG) --target $(TARGET)
	@rm src/linker.ld

# Kernel build with M-Mode support (for -bios none)
kernel-nobios:
	@echo "Platform: $(BOARD) with M-Mode SBI - no external bootloader [$(ARCH)]"
	@cp src/linker-$(BOARD)-nobios.ld src/linker.ld
	@cargo build $(MODE_ARG) --target $(TARGET) --features nobios
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

# Original QEMU args with rustsbi bootloader (riscv64 only)
QEMU_ARGS := -machine virt \
			 -nographic \
			 -bios $(BOOTLOADER) \
			 -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) \
			 -drive file=$(FS_IMG),if=none,format=raw,id=x0 \
			 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

# New QEMU args with -bios none (uses our M-Mode implementation)
QEMU_ARGS_NOBIOS := -machine virt \
			 -nographic \
			 -bios none \
			 -device loader,file=$(KERNEL_BIN),addr=$(M_MODE_ENTRY_PA) \
			 -drive file=$(FS_IMG),if=none,format=raw,id=x0 \
			 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

qemu-version-check:
	@sh scripts/qemu-ver-check.sh $(QEMU_PATH)

run-inner: qemu-version-check build
ifeq ($(ARCH), riscv32)
	$(error RustSBI mode is only supported for riscv64. Use 'make run-nobios ARCH=riscv32' instead.)
endif
	@$(QEMU_PATH) $(QEMU_ARGS)

# Run with -bios none (M-Mode implementation)
run-nobios: qemu-version-check build-nobios
	@$(QEMU_PATH) $(QEMU_ARGS_NOBIOS)

debug: qemu-version-check build
	@tmux new-session -d \
		"$(QEMU_PATH) $(QEMU_ARGS) -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch $(GDB_ARCH)' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

debug-nobios: qemu-version-check build-nobios
	@tmux new-session -d \
		"$(QEMU_PATH) $(QEMU_ARGS_NOBIOS) -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch $(GDB_ARCH)' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

gdbserver: qemu-version-check build
	@$(QEMU_PATH) $(QEMU_ARGS) -s -S

gdbserver-nobios: qemu-version-check build-nobios
	@$(QEMU_PATH) $(QEMU_ARGS_NOBIOS) -s -S

gdbclient:
	@riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch $(GDB_ARCH)' -ex 'target remote localhost:1234'

.PHONY: build build-nobios env kernel kernel-nobios clean disasm disasm-vim run-inner run-nobios gdbserver gdbserver-nobios gdbclient qemu-version-check debug debug-nobios fs-img fs-img-nobios
