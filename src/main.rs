#![no_main]
#![no_std]

use core::panic::PanicInfo;
use rcpu_os::println;

// The new entry point
#[no_mangle] // Disables mangling the name of _start
pub extern "C" fn _start() -> ! {
    println!("Hello RCPU_{}S", 0);

    rcpu_os::init();

    fn stack_overflow() {
        stack_overflow(); // for each recursion, the return address is pushed
    }

    stack_overflow();

    println!("It did not crash!");

    loop{}
}

// Function that gets called on panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop{}
}
