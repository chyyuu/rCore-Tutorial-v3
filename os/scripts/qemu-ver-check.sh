#!/bin/bash
# QEMU version check script

QEMU=$1

if [ -z "$QEMU" ]; then
    echo "Usage: $0 <qemu-binary>"
    exit 1
fi

# Check if QEMU is in PATH or at common locations
if command -v $QEMU &> /dev/null; then
    QEMU_PATH=$QEMU
elif [ -x "/home/chyyuu/thecodes/qemu-7.0.0/build/riscv64-softmmu/$QEMU" ]; then
    QEMU_PATH="/home/chyyuu/thecodes/qemu-7.0.0/build/riscv64-softmmu/$QEMU"
elif [ -x "/home/chyyuu/thecodes/qemu-7.0.0/build/riscv32-softmmu/$QEMU" ]; then
    QEMU_PATH="/home/chyyuu/thecodes/qemu-7.0.0/build/riscv32-softmmu/$QEMU"
else
    # Try to find it
    QEMU_PATH=$(which $QEMU 2>/dev/null || find /home/chyyuu -name "$QEMU" -type f -executable 2>/dev/null | head -1)
    if [ -z "$QEMU_PATH" ]; then
        echo "Warning: $QEMU not found, but continuing..."
        exit 0
    fi
fi

# Get QEMU version
VERSION=$($QEMU_PATH --version 2>/dev/null | head -1 | grep -oE '[0-9]+\.[0-9]+' | head -1)

if [ -z "$VERSION" ]; then
    echo "Warning: Could not determine QEMU version"
    exit 0
fi

MAJOR=$(echo $VERSION | cut -d. -f1)
MINOR=$(echo $VERSION | cut -d. -f2)

# Require QEMU 5.0 or later
if [ "$MAJOR" -lt 5 ]; then
    echo "Warning: QEMU version $VERSION may be too old. Recommend 5.0+"
fi

exit 0
