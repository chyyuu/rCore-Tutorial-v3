//! Console driver for text output
//! Supports both SBI-based output (with RustSBI) and direct UART access (nobios mode)

use core::fmt::{self, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        #[cfg(feature = "nobios")]
        {
            // Direct UART access in nobios mode
            use crate::drivers::uart;
            for c in s.chars() {
                uart::putchar(c as u8);
            }
        }
        #[cfg(not(feature = "nobios"))]
        {
            // SBI-based output (with RustSBI)
            use crate::sbi::console_putchar;
            for c in s.chars() {
                console_putchar(c as usize);
            }
        }
        Ok(())
    }
}

/// Print formatted string
pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
/// print string macro
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?))
    }
}

#[macro_export]
/// println string macro with timestamp
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(
            concat!("[{:>5} ms] ", $fmt, "\n"),
            $crate::timer::get_time_ms()
            $(, $($arg)+)?
        ))
    }
}
