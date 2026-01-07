#!/bin/bash

QEMU=$1
VERSION=$($QEMU --version | head -n1 | sed 's/.*version \([0-9]*\).*/\1/')
if [ "$VERSION" -ge 7 ]; then
    echo -e "\033[0;32mQEMU version is $($QEMU --version | head -n1 | sed 's/.*version \([0-9.]*\).*/\1/')(>=7), OK!\033[0m"
    exit 0
else
    echo -e "\033[0;31mQEMU version is $VERSION, please upgrade to >= 7.0.0\033[0m"
    exit 1
fi
