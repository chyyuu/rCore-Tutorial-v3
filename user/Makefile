# Architecture selection: riscv32 or riscv64 (default)
ARCH ?= riscv64

ifeq ($(ARCH), riscv32)
    TARGET := riscv32imac-unknown-none-elf
    OBJDUMP := rust-objdump --arch-name=riscv32
    OBJCOPY := rust-objcopy --binary-architecture=riscv32
else
    TARGET := riscv64gc-unknown-none-elf
    OBJDUMP := rust-objdump --arch-name=riscv64
    OBJCOPY := rust-objcopy --binary-architecture=riscv64
endif

MODE := release
APP_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
APPS := $(wildcard $(APP_DIR)/*.rs)
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))
BINS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%.bin, $(APPS))

elf: $(APPS)
	@python3 build.py $(TARGET)

binary: elf
	$(foreach elf, $(ELFS), $(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.bin, $(elf));)

build: binary

clean:
	@cargo clean

.PHONY: elf binary build clean
