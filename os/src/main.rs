#![no_std]
#![no_main]
#![allow(warnings)]

extern crate alloc;

#[macro_use]
extern crate bitflags;

use log::*;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod config;
mod drivers;
mod fs;
mod lang_items;
mod logging;
mod mm;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

// M-Mode SBI implementation (only used when booting with -bios none)
#[cfg(feature = "nobios")]
mod msbi;

// Include M-Mode entry point (for -bios none boot)
#[cfg(all(feature = "nobios", target_pointer_width = "64"))]
core::arch::global_asm!(include_str!("m_entry_rv64.asm"));

#[cfg(all(feature = "nobios", target_pointer_width = "32"))]
core::arch::global_asm!(include_str!("m_entry_rv32.asm"));

// Include S-Mode entry point
core::arch::global_asm!(include_str!("entry.asm"));

/// Clear BSS segment
fn clear_bss() {
    unsafe extern "C" {
        safe fn sbss();
        safe fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

#[unsafe(no_mangle)]
/// The rust entry-point of os
pub fn rust_main() -> ! {
    clear_bss();
    // Initialize UART driver for direct hardware access in nobios mode
    #[cfg(feature = "nobios")]
    {
        drivers::uart::init();
        println!("[kernel] UART driver initialized (direct mode)");
    }
    logging::init();
    println!("[kernel] Hello, world!");
    trace!("[kernel] .text section loaded");
    debug!("[kernel] .rodata section loaded");
    info!("[kernel] .data section loaded");
    warn!("[kernel] .bss section cleared");
    mm::init();
    mm::remap_test();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    fs::list_apps();
    task::add_initproc();
    info!("after initproc!");
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
