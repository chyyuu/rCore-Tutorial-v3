//! Implementation of [`TrapContext`]
use riscv::register::sstatus::{self, Sstatus};

#[repr(C)]
/// trap context structure containing sstatus, sepc and registers
pub struct TrapContext {
    /// general regs[0..31]
    pub x: [usize; 32],
    /// CSR sstatus      
    pub sstatus: Sstatus,
    /// CSR sepc
    pub sepc: usize,
    /// Addr of Page Table
    pub kernel_satp: usize,
    /// kernel stack
    pub kernel_sp: usize,
    /// Addr of trap_handler function
    pub trap_handler: usize,
}

impl TrapContext {
    /// set stack pointer to x_2 reg (sp)
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    /// init app context
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let sstatus = sstatus::read();
        // set CPU privilege to User after trapping back
        // Note: In newer riscv crate, we need to use unsafe to modify sstatus
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
            kernel_satp,
            kernel_sp,
            trap_handler,
        };
        // Set SPP to User mode by clearing the bit directly in the sstatus bits
        // SPP is bit 8, we need to clear it for User mode
        unsafe {
            let sstatus_bits = &mut cx.sstatus as *mut Sstatus as *mut usize;
            *sstatus_bits &= !(1 << 8); // Clear SPP bit (bit 8) for User mode
        }
        cx.set_sp(sp);
        cx
    }
}
