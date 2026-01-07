#!/bin/bash
# QEMU version check script

QEMU=$1

if [ -z "$QEMU" ]; then
    echo "Usage: $0 <qemu-binary>"
    exit 1
fi

# Check if QEMU is in PATH or is an absolute/relative path that exists
if ! command -v "$QEMU" &> /dev/null && [ ! -x "$QEMU" ]; then
    echo "Error: $QEMU not found or not executable"
    exit 1
fi

# Get QEMU version
VERSION=$($QEMU --version | head -1 | grep -oE '[0-9]+\.[0-9]+' | head -1)

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


