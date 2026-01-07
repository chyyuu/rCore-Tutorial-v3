//!Stdin & Stdout
use super::File;
use crate::mm::UserBuffer;
use crate::task::suspend_current_and_run_next;

///Standard input
pub struct Stdin;
///Standard output
pub struct Stdout;

/// Read a character from console
/// In nobios mode, directly access UART hardware
/// Otherwise, use SBI call
fn console_getchar() -> Option<u8> {
    #[cfg(feature = "nobios")]
    {
        use crate::drivers::uart;
        uart::getchar()
    }
    #[cfg(not(feature = "nobios"))]
    {
        use crate::sbi;
        let c = sbi::console_getchar();
        if c == 0 || c == usize::MAX {
            None
        } else {
            Some(c as u8)
        }
    }
}

impl File for Stdin {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
    fn read(&self, mut user_buf: UserBuffer) -> usize {
        assert_eq!(user_buf.len(), 1);
        // busy loop
        let ch: u8;
        loop {
            if let Some(c) = console_getchar() {
                ch = c;
                break;
            } else {
                suspend_current_and_run_next();
            }
        }
        unsafe {
            user_buf.buffers[0].as_mut_ptr().write_volatile(ch);
        }
        1
    }
    fn write(&self, _user_buf: UserBuffer) -> usize {
        panic!("Cannot write to stdin!");
    }
}

impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, _user_buf: UserBuffer) -> usize {
        panic!("Cannot read from stdout!");
    }
    fn write(&self, user_buf: UserBuffer) -> usize {
        for buffer in user_buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        user_buf.len()
    }
}
