//! The main module and entrypoint
//!
//! The operating system and app also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality [`clear_bss()`]. (See its source code for
//! details.)
//!
//! We then call [`println!`] to display `Hello, world!`.

#![deny(missing_docs)]
#![allow(dead_code)]
#![no_std]
#![no_main]

use core::arch::global_asm;

#[macro_use]
mod console;
mod lang_items;
mod logging;
mod sbi;

// M-Mode SBI implementation (only used when booting with -bios none)
#[cfg(feature = "nobios")]
mod msbi;

// Include M-Mode entry point (for -bios none boot)
#[cfg(feature = "nobios")]
global_asm!(include_str!("m_entry.asm"));

// Include S-Mode entry point
global_asm!(include_str!("entry.asm"));

/// clear BSS segment
pub fn clear_bss() {
    unsafe extern "C" {
        safe fn sbss();
        safe fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

/// Direct SBI call for testing
fn direct_putchar(c: u8) {
    unsafe {
        core::arch::asm!(
            "li a7, 1",      // EID = 1 (legacy console_putchar)
            "mv a0, {0}",    // character to print
            "ecall",
            in(reg) c as usize,
            out("a0") _,
            out("a1") _,
            out("a7") _,
        );
    }
}

/// the rust entry-point of os
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    
    // Test with direct SBI calls
    direct_putchar(b'H');
    direct_putchar(b'e');
    direct_putchar(b'l');
    direct_putchar(b'l');
    direct_putchar(b'o');
    direct_putchar(b'\n');
    
    sbi::shutdown(false)
}
