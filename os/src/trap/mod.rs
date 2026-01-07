mod context;

use crate::config::TRAMPOLINE;
use crate::syscall::syscall;
use crate::task::{
    SignalFlags, check_signals_of_current, current_add_signal, current_trap_cx,
    current_trap_cx_user_va, current_user_token, exit_current_and_run_next,
    suspend_current_and_run_next,
};
use crate::timer::{check_timer, set_next_trigger};
use core::arch::{asm, global_asm};
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Trap},
    sie, stval, stvec,
};

#[cfg(target_pointer_width = "64")]
global_asm!(include_str!("trap_rv64.S"));
#[cfg(target_pointer_width = "32")]
global_asm!(include_str!("trap_rv32.S"));

// Exception codes
const EXCEPTION_USER_ECALL: usize = 8;
const EXCEPTION_STORE_FAULT: usize = 7;
const EXCEPTION_STORE_PAGE_FAULT: usize = 15;
const EXCEPTION_INSTRUCTION_FAULT: usize = 1;
const EXCEPTION_INSTRUCTION_PAGE_FAULT: usize = 12;
const EXCEPTION_LOAD_FAULT: usize = 5;
const EXCEPTION_LOAD_PAGE_FAULT: usize = 13;
const EXCEPTION_ILLEGAL_INSTRUCTION: usize = 2;

// Interrupt codes
const INTERRUPT_SUPERVISOR_TIMER: usize = 5;

pub fn init() {
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[unsafe(no_mangle)]
/// handle an interrupt, exception, or system call from user space
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let scause = scause::read();
    let stval = stval::read();
    let cause = scause.cause();
    
    match cause {
        Trap::Exception(code) => {
            match code {
                EXCEPTION_USER_ECALL => {
                    // jump to next instruction anyway
                    let mut cx = current_trap_cx();
                    cx.sepc += 4;
                    // get system call return value
                    let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]);
                    // cx is changed during sys_exec, so we have to call it again
                    cx = current_trap_cx();
                    cx.x[10] = result as usize;
                }
                EXCEPTION_STORE_FAULT
                | EXCEPTION_STORE_PAGE_FAULT
                | EXCEPTION_INSTRUCTION_FAULT
                | EXCEPTION_INSTRUCTION_PAGE_FAULT
                | EXCEPTION_LOAD_FAULT
                | EXCEPTION_LOAD_PAGE_FAULT => {
                    /*
                    println!(
                        "[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                        stval,
                        current_trap_cx().sepc,
                    );
                    */
                    current_add_signal(SignalFlags::SIGSEGV);
                }
                EXCEPTION_ILLEGAL_INSTRUCTION => {
                    current_add_signal(SignalFlags::SIGILL);
                }
                _ => {
                    panic!(
                        "Unsupported exception {:?}, stval = {:#x}!",
                        cause,
                        stval
                    );
                }
            }
        }
        Trap::Interrupt(code) => {
            match code {
                INTERRUPT_SUPERVISOR_TIMER => {
                    set_next_trigger();
                    check_timer();
                    suspend_current_and_run_next();
                }
                _ => {
                    panic!(
                        "Unsupported interrupt {:?}, stval = {:#x}!",
                        cause,
                        stval
                    );
                }
            }
        }
    }
    // check signals
    if let Some((errno, msg)) = check_signals_of_current() {
        println!("[kernel] {}", msg);
        exit_current_and_run_next(errno);
    }
    trap_return();
}

#[unsafe(no_mangle)]
/// set the new addr of __restore asm function in TRAMPOLINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_user_va = current_trap_cx_user_va();
    let user_satp = current_user_token();
    unsafe extern "C" {
        unsafe fn __alltraps();
        unsafe fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_user_va,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}

#[unsafe(no_mangle)]
/// Unimplement: traps/interrupts/exceptions from kernel mode
/// Todo: Chapter 9: I/O device
pub fn trap_from_kernel() -> ! {
    use riscv::register::sepc;
    println!("stval = {:#x}, sepc = {:#x}", stval::read(), sepc::read());
    panic!("a trap {:?} from kernel!", scause::read().cause());
}

pub use context::TrapContext;
