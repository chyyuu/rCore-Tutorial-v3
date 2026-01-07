//! UART driver for 16550 compatible UART
//! This driver allows S-Mode kernel to directly access UART hardware
//! without going through SBI calls

use core::fmt::{self, Write};

/// UART base address on QEMU virt machine
const UART_BASE: usize = 0x1000_0000;

/// UART register offsets
const UART_RHR: usize = 0; // Receive Holding Register (read)
const UART_THR: usize = 0; // Transmit Holding Register (write)
const UART_IER: usize = 1; // Interrupt Enable Register
const UART_FCR: usize = 2; // FIFO Control Register (write)
#[allow(dead_code)]
const UART_ISR: usize = 2; // Interrupt Status Register (read)
const UART_LCR: usize = 3; // Line Control Register
const UART_MCR: usize = 4; // Modem Control Register
const UART_LSR: usize = 5; // Line Status Register
#[allow(dead_code)]
const UART_MSR: usize = 6; // Modem Status Register
#[allow(dead_code)]
const UART_SPR: usize = 7; // Scratch Pad Register

/// Line Status Register bits
const LSR_DATA_READY: u8 = 1 << 0; // Data ready
const LSR_THR_EMPTY: u8 = 1 << 5;  // Transmit-hold-register empty

/// UART driver structure
pub struct Uart {
    base_addr: usize,
}

impl Uart {
    /// Create a new UART instance
    pub const fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    /// Read a byte from the specified offset
    #[inline]
    fn read(&self, offset: usize) -> u8 {
        unsafe { ((self.base_addr + offset) as *const u8).read_volatile() }
    }

    /// Write a byte to the specified offset
    #[inline]
    fn write(&self, offset: usize, value: u8) {
        unsafe { ((self.base_addr + offset) as *mut u8).write_volatile(value) }
    }

    /// Initialize the UART
    pub fn init(&mut self) {
        // Disable all interrupts
        self.write(UART_IER, 0x00);

        // Enable DLAB (Divisor Latch Access Bit) to set baud rate
        self.write(UART_LCR, 0x80);

        // Set baud rate divisor to 3 (38.4K baud for QEMU)
        self.write(UART_RHR, 0x03); // Divisor latch LSB
        self.write(UART_IER, 0x00); // Divisor latch MSB

        // Disable DLAB and set word length to 8 bits, no parity, 1 stop bit
        self.write(UART_LCR, 0x03);

        // Enable FIFO, clear TX/RX queues with 14-byte threshold
        self.write(UART_FCR, 0xC7);

        // Enable auxiliary output 2 (used for interrupts on some hardware)
        // Also set RTS and DTR
        self.write(UART_MCR, 0x0B);

        // Enable receive interrupts (optional, for interrupt-driven I/O)
        // For now, we'll use polling mode
        self.write(UART_IER, 0x00);
    }

    /// Write a byte to UART, waiting if necessary
    pub fn put(&mut self, byte: u8) {
        // Wait for THR to be empty
        while self.read(UART_LSR) & LSR_THR_EMPTY == 0 {
            core::hint::spin_loop();
        }
        self.write(UART_THR, byte);
    }

    /// Write a string to UART
    pub fn puts(&mut self, s: &str) {
        for byte in s.bytes() {
            self.put(byte);
        }
    }

    /// Try to read a byte from UART (non-blocking)
    /// Returns Some(byte) if data is available, None otherwise
    pub fn get(&mut self) -> Option<u8> {
        if self.read(UART_LSR) & LSR_DATA_READY != 0 {
            Some(self.read(UART_RHR))
        } else {
            None
        }
    }

    /// Read a byte from UART (blocking)
    #[allow(dead_code)]
    pub fn get_blocking(&mut self) -> u8 {
        loop {
            if let Some(c) = self.get() {
                return c;
            }
            core::hint::spin_loop();
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.puts(s);
        Ok(())
    }
}

/// Global UART instance
static mut UART: Uart = Uart::new(UART_BASE);

/// Initialize the global UART
pub fn init() {
    unsafe {
        let uart = &mut *core::ptr::addr_of_mut!(UART);
        uart.init();
    }
}

/// Write a character to UART
pub fn putchar(c: u8) {
    unsafe {
        let uart = &mut *core::ptr::addr_of_mut!(UART);
        uart.put(c);
    }
}

/// Read a character from UART (non-blocking)
pub fn getchar() -> Option<u8> {
    unsafe {
        let uart = &mut *core::ptr::addr_of_mut!(UART);
        uart.get()
    }
}

/// Read a character from UART (blocking)
#[allow(dead_code)]
pub fn getchar_blocking() -> u8 {
    unsafe {
        let uart = &mut *core::ptr::addr_of_mut!(UART);
        uart.get_blocking()
    }
}

/// Write formatted output to UART
#[allow(dead_code)]
pub fn _print(args: fmt::Arguments) {
    unsafe {
        let uart = &mut *core::ptr::addr_of_mut!(UART);
        uart.write_fmt(args).unwrap();
    }
}


