//! Constants used in rCore for qemu

/// Clock frequency
pub const CLOCK_FREQ: usize = 12500000;

/// Memory end address
pub const MEMORY_END: usize = 0x81000000;

/// MMIO regions
pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC in virt machine
];

//ref:: https://github.com/andre-richter/qemu-exit
use core::arch::asm;

const EXIT_SUCCESS: u32 = 0x5555; // Equals `exit(0)`. qemu successful exit
const EXIT_FAILURE_FLAG: u32 = 0x3333;
const EXIT_FAILURE: u32 = exit_code_encode(1); // Equals `exit(1)`. qemu failed exit
const EXIT_RESET: u32 = 0x7777; // qemu reset

/// QEMU exit trait
pub trait QEMUExit {
    /// Exit with specified return code.
    fn exit(&self, code: u32) -> !;

    /// Exit QEMU using `EXIT_SUCCESS`, aka `0`, if possible.
    fn exit_success(&self) -> !;

    /// Exit QEMU using `EXIT_FAILURE`, aka `1`.
    #[allow(dead_code)]
    fn exit_failure(&self) -> !;
}

/// RISC-V configuration (works for both RV32 and RV64)
pub struct RISCV {
    /// Address of the sifive_test mapped device.
    addr: usize,
}

/// Encode the exit code using EXIT_FAILURE_FLAG.
const fn exit_code_encode(code: u32) -> u32 {
    (code << 16) | EXIT_FAILURE_FLAG
}

impl RISCV {
    /// Create an instance.
    pub const fn new(addr: usize) -> Self {
        RISCV { addr }
    }
}

impl QEMUExit for RISCV {
    /// Exit qemu with specified exit code.
    fn exit(&self, code: u32) -> ! {
        // If code is not a special value, we need to encode it with EXIT_FAILURE_FLAG.
        let code_new = match code {
            EXIT_SUCCESS | EXIT_FAILURE | EXIT_RESET => code,
            _ => exit_code_encode(code),
        };

        unsafe {
            asm!(
                "sw {0}, 0({1})",
                in(reg) code_new,
                in(reg) self.addr
            );

            // For the case that the QEMU exit attempt did not work, transition into an infinite
            // loop.
            loop {
                asm!("wfi", options(nomem, nostack));
            }
        }
    }

    fn exit_success(&self) -> ! {
        self.exit(EXIT_SUCCESS);
    }

    fn exit_failure(&self) -> ! {
        self.exit(EXIT_FAILURE);
    }
}

const VIRT_TEST: usize = 0x100000;

/// QEMU exit handle
pub const QEMU_EXIT_HANDLE: RISCV = RISCV::new(VIRT_TEST);
