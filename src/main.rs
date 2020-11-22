// Disable standard library and "regular" entry point
#![no_std]
#![no_main]

use core::panic::PanicInfo;

extern crate volatile;
extern crate lazy_static;
extern crate spin;

mod terminal;

// The new entry point
#[no_mangle] // Disables mangling the name of _start
pub extern "C" fn _start() -> ! {
    println!("Hello RCPU_{}S", 0);

    loop{}
}

// Function that gets called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}
