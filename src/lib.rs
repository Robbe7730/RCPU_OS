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
mod memory;

use core::panic::PanicInfo;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    println!("Hello RCPU_{}S", 1);

    init();

    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };

    let memory_tag = boot_info.memory_map_tag().expect("No memory map tag found");

    // Find the first memory area big enoug to hold the RCPU memory space
    // memory_areas only shows available memory areas
    // TODO: this could probably do with better use of paging
    let working_space_start = memory_tag.memory_areas().fold(
        None,
        |_acc, memory_area|
            if memory_area.size() >= 0xffff {
                Some(memory_area.start_address())
            } else {
                None
            }
        ).expect("No available memory found");

    // For now, just select the first module
    let mut running_program = rcpu::RCPUProgram::from_module_tag(
        boot_info.module_tags().next().expect("No modules found!"),
        working_space_start as usize
    );


    while running_program.running {
        running_program.step()
    }

    // Halt the processor
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

