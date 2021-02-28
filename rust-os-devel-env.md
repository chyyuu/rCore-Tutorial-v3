
# in ubuntu 20.04 x86-64
## rust
```
export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
curl https://sh.rustup.rs -sSf | sh

rustup target add riscv64gc-unknown-none-elf
cargo install cargo-binutils
rustup component add llvm-tools-preview
rustup component add rust-src
```

## qemu
```
sudo apt install autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev \
              gawk build-essential bison flex texinfo gperf libtool patchutils bc \
              zlib1g-dev libexpat-dev git pkg-config  libglib2.0-dev libpixman-1-dev tmux

              
wget https://download.qemu.org/qemu-5.0.0.tar.xz

wget https://download.qemu.org/qemu-5.0.0.tar.xz
# 解压
tar xvJf qemu-5.0.0.tar.xz
# 编译安装并配置 RISC-V 支持
cd qemu-5.0.0
./configure --target-list=riscv64-softmmu,riscv64-linux-user
make -j$(nproc)
make install
```

## riscv64 gnu tools
```
GCC 10.1, with RVV-1.0 intrinsic supported
Binutils 2.35, with RVV-1.0 instructions supported
Newlib 3.2, with string and memory functions optimized for performance
Gdb 9.1 with RVV-1.0 debug information supported
QEMU 5.1.0, with RVV-1.0 support
Spike-DASM 1.0.0
OpenOCD 0.10.0 latest commit
Trace-Decoder 0.9.2

Prebuilt Binary Packages:
https://github.com/sifive/freedom-tools/releases
https://static.dev.sifive.com/dev-tools/freedom-tools/v2020.08/riscv64-unknown-elf-gcc-10.1.0-2020.08.2-x86_64-linux-ubuntu14.tar.gz
https://static.dev.sifive.com/dev-tools/freedom-tools/v2020.08/riscv-openocd-0.10.0-2020.08.1-x86_64-linux-ubuntu14.tar.gz
https://static.dev.sifive.com/dev-tools/freedom-tools/v2020.08/riscv-qemu-5.1.0-2020.08.1-x86_64-linux-ubuntu14.tar.gz
https://static.dev.sifive.com/dev-tools/freedom-tools/v2020.08/sdk-utilities-1.0.0-2020.08.1-x86_64-linux-ubuntu14.tar.gz
https://static.dev.sifive.com/dev-tools/freedom-tools/v2020.08/trace-decoder-0.9.2-2020.08.2-x86_64-linux-ubuntu14.tar.gz



wget https://static.dev.sifive.com/dev-tools/freedom-tools/v2020.08/riscv64-unknown-elf-gcc-10.1.0-2020.08.2-x86_64-linux-ubuntu14.tar.gz
cd /usr/local
tar zxf riscv64-unknown-elf-gcc-10.1.0-2020.08.2-x86_64-linux-ubuntu14.tar.gz


```
## set env var
```
# add Below Lines to ~/.bashrc

source "$HOME/.cargo/env"
export PATH=$PATH:/usr/local/riscv64-unknown-elf-gcc-10.1.0-2020.08.2-x86_64-linux-ubuntu14/bin

```
