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
mod rcpu;

use core::panic::PanicInfo;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    println!("Hello RCPU_{}S", 1);

    init();

    println!("multiboot_information_address: {}", multiboot_information_address);

    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };

    // Separate iterator because count() consumes it
    let num_modules = boot_info.module_tags().count();

    let module_tags = boot_info.module_tags();

    println!("Modules available ({}):", num_modules);
    for module_tag in module_tags {
        println!("{}: {}-{}", module_tag.name(), module_tag.start_address(),
            module_tag.end_address());
    }

    // For now, just select the first module
    let running_program = rcpu::RCPUProgram::from_module_tag(boot_info.module_tags().next().expect("No modules found!"));

    println!("Value: {:#06x}", running_program.read(0));

    running_program.write(0, 0x1337);

    println!("Value: {:#06x}", running_program.read(0));

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

