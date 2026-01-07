//! Trap handling functionality
//!
//! For rCore, we have a single trap entry point, namely `__alltraps`. At
//! initialization in [`init()`], we set the `stvec` CSR to point to it.
//!
//! All traps go through `__alltraps`, which is defined in `trap.S`. The
//! assembly language code does just enough work restore the kernel space
//! context, ensuring that Rust code safely runs, and transfers control to
//! [`trap_handler()`].
//!
//! It then calls different functionality based on what exactly the exception
//! was. For example, timer interrupts trigger task preemption, and syscalls go
//! to [`syscall()`].

mod context;

use crate::syscall::syscall;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::set_next_trigger;
use core::arch::global_asm;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Trap},
    sie, stval, stvec,
};

// Select architecture-specific trap assembly file
#[cfg(target_pointer_width = "64")]
global_asm!(include_str!("trap_rv64.S"));

#[cfg(target_pointer_width = "32")]
global_asm!(include_str!("trap_rv32.S"));

/// initialize CSR `stvec` as the entry of `__alltraps`
pub fn init() {
    unsafe extern "C" {
        safe fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

/// timer interrupt enabled
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

// Exception codes from RISC-V spec
const EXCEPTION_USER_ECALL: usize = 8;
const EXCEPTION_STORE_FAULT: usize = 7;
const EXCEPTION_STORE_PAGE_FAULT: usize = 15;
const EXCEPTION_ILLEGAL_INSTRUCTION: usize = 2;

// Interrupt codes
const INTERRUPT_SUPERVISOR_TIMER: usize = 5;

#[unsafe(no_mangle)]
/// handle an interrupt, exception, or system call from user space
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
    match scause.cause() {
        Trap::Exception(EXCEPTION_USER_ECALL) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(EXCEPTION_STORE_FAULT) | Trap::Exception(EXCEPTION_STORE_PAGE_FAULT) => {
            println!("[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.", stval, cx.sepc);
            exit_current_and_run_next();
        }
        Trap::Exception(EXCEPTION_ILLEGAL_INSTRUCTION) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit_current_and_run_next();
        }
        Trap::Interrupt(INTERRUPT_SUPERVISOR_TIMER) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    cx
}

pub use context::TrapContext;
