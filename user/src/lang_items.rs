#[panic_handler]
fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
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
