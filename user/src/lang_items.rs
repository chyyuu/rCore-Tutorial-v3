//! Panic handler for user applications

use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    if let Some(location) = panic_info.location() {
        println!(
            "Panicked at {}:{}, {:?}",
            location.file(),
            location.line(),
            panic_info.message()
        );
    } else {
        println!("Panicked: {:?}", panic_info.message());
    }
    loop {}
}
