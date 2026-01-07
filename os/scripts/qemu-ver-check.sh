#!/bin/bash
# QEMU version check script

QEMU=$1

if [ -z "$QEMU" ]; then
    echo "Usage: $0 <qemu-binary>"
    exit 1
fi

# Check if QEMU is an absolute path or in PATH
if [ -x "$QEMU" ]; then
    QEMU_PATH="$QEMU"
elif command -v "$QEMU" &> /dev/null; then
    QEMU_PATH=$(command -v "$QEMU")
else
    echo "Error: $QEMU not found"
    exit 1
fi

# Get QEMU version
VERSION=$($QEMU_PATH --version | head -1 | grep -oE '[0-9]+\.[0-9]+' | head -1)

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
