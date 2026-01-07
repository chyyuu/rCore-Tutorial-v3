#!/bin/bash
# Simple test for keyboard input in nobios mode

cd /home/chyyuu/thecodes/rCore-Tutorial-v3/os

echo "Building kernel in nobios mode for riscv64..."
make build-nobios ARCH=riscv64 > /dev/null 2>&1

echo "========================================="
echo "Starting QEMU with nobios mode..."
echo "You should see 'Rust user shell' prompt."
echo "Type 'ls' and press Enter to list apps."
echo "Type 'exit' to quit the shell."
echo "Press Ctrl+A then X to exit QEMU."
echo "========================================="
echo ""

# Send 'ls\n' and 'exit\n' via stdin
echo -e "ls\nexit" | timeout 15 /home/chyyuu/thecodes/qemu-7.0.0/build/riscv64-softmmu/qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios none \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80000000 \
    -drive file=../user/target/riscv64gc-unknown-none-elf/release/fs.img,if=none,format=raw,id=x0 \
    -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

echo ""
echo "========================================="
echo "Test completed!"


