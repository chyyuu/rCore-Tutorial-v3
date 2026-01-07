#!/bin/bash
# Test keyboard input in nobios mode

cd /home/chyyuu/thecodes/rCore-Tutorial-v3/os

echo "Building kernel in nobios mode for riscv64..."
make build-nobios ARCH=riscv64 > /dev/null 2>&1

echo "Starting QEMU with nobios mode..."
echo "Will send 'ls' command after 2 seconds..."

# Create a temporary expect script
cat > /tmp/test_input.exp << 'EOF'
#!/usr/bin/expect -f
set timeout 10

spawn /home/chyyuu/thecodes/qemu-7.0.0/build/riscv64-softmmu/qemu-system-riscv64 -machine virt -nographic -bios none -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80000000 -drive file=../user/target/riscv64gc-unknown-none-elf/release/fs.img,if=none,format=raw,id=x0 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

expect ">> "
send "ls\r"
expect ">> "
send "exit\r"
expect eof
EOF

chmod +x /tmp/test_input.exp
/tmp/test_input.exp

echo "Test completed!"
rm /tmp/test_input.exp


