// Disable standard library and "regular" entry point
#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Function that gets called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}

// The new entry point, "no_mangle" disables mangling the name of _start
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop{}
}
