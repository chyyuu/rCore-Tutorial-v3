pub const CLOCK_FREQ: usize = 12500000;

#[cfg(target_pointer_width = "64")]
pub const MEMORY_END: usize = 0x8800_0000;
#[cfg(target_pointer_width = "32")]
pub const MEMORY_END: usize = 0x8800_0000;

/// MMIO regions
pub const MMIO: &[(usize, usize)] = &[
    (0x1000_0000, 0x1000), // UART16550 in virt machine
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC in virt machine
    (0x1000_1000, 0x00_1000), // Virtio Block in virt machine
];

pub type BlockDeviceImpl = crate::drivers::block::VirtIOBlock;

// QEMU exit codes
#[allow(dead_code)]
const EXIT_SUCCESS: u32 = 0x5555;
#[allow(dead_code)]
const EXIT_FAILURE: u32 = 0x3333;
#[allow(dead_code)]
const EXIT_RESET: u32 = 0x7777;

/// QEMU exit functionality
pub trait QEMUExit {
    /// Exit with specified code
    fn exit(&self, code: u32) -> !;

    /// Exit QEMU using `EXIT_SUCCESS`, aka `0x5555`.
    fn exit_success(&self) -> !;

    /// Exit QEMU using `EXIT_FAILURE`, aka `1`.
    #[allow(dead_code)]
    fn exit_failure(&self) -> !;

    /// Reset QEMU using `EXIT_RESET`.
    fn exit_reset(&self) -> !;
}

/// RISCV64 configuration
pub struct RISCV64 {
    /// Address of the sifive_test mapped device.
    addr: u64,
}

/// Encode the exit code using EXIT_FAILURE_FLAG.
const fn exit_code_encode(code: u32) -> u32 {
    (code << 16) | 0x3333
}

impl RISCV64 {
    /// Create an instance.
    pub const fn new(addr: u64) -> Self {
        RISCV64 { addr }
    }
}

impl QEMUExit for RISCV64 {
    /// Exit qemu with specified exit code.
    fn exit(&self, code: u32) -> ! {
        // If code is a special value, don't encode it,
        // otherwise, encode it using EXIT_FAILURE_FLAG.
        let code_new = match code {
            EXIT_SUCCESS | EXIT_FAILURE | EXIT_RESET => code,
            _ => exit_code_encode(code),
        };
        
        let addr = self.addr as usize;
        unsafe {
            core::arch::asm!(
                "sw {0}, 0({1})",
                in(reg)code_new, in(reg)addr
            );

            // For the case that the QEMU exit attempt did not work, transition into an infinite
            // loop.292      //292292 292292292292292     292292292292292 292292292     292292
            loop {
                core::arch::asm!("wfi", options(nomem, nostack));
            }
        }
    }

    fn exit_success(&self) -> ! {
        self.exit(EXIT_SUCCESS);
    }

    fn exit_failure(&self) -> ! {
        self.exit(EXIT_FAILURE);
    }

    fn exit_reset(&self) -> ! {
        self.exit(EXIT_RESET);
    }
}

const VIRT_TEST: u64 = 0x100000;

/// QEMU exit handle
pub const QEMU_EXIT_HANDLE: RISCV64 = RISCV64::new(VIRT_TEST);
