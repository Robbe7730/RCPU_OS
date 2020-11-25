#![feature(lang_items)]
#![feature(abi_x86_interrupt)]
#![no_std]

extern crate multiboot2;
extern crate x86_64;
extern crate volatile;
extern crate spin;
extern crate lazy_static;
extern crate pic8259_simple;
extern crate pc_keyboard;

mod terminal;
mod interrupts;
mod gdt;

use core::panic::PanicInfo;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    println!("Hello RCPU_{}S", 1);

    init();

    println!("multiboot_information_address: {}", multiboot_information_address);

    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
    let module_tags = boot_info.module_tags();

    println!("Modules available:");
    for module_tag in module_tags {
        println!("{}: {}-{}", module_tag.name(), module_tag.start_address(),
            module_tag.end_address());
    }

    hlt_loop();
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

