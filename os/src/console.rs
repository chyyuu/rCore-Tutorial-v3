//! SBI console driver, for text output

use crate::sbi::console_putchar;
use core::fmt::{self, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

/// Print without timestamp (internal use)
pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

/// Print with timestamp prefix
pub fn print_with_time(args: fmt::Arguments) {
    use crate::timer::get_time_ms;
    let time_ms = get_time_ms();
    Stdout.write_fmt(format_args!("[{:>5} ms] ", time_ms)).unwrap();
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
/// print string macro (without timestamp, for internal/raw output)
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
/// println string macro with timestamp
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print_with_time(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
