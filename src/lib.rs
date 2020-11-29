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
mod keyboard;

use core::panic::PanicInfo;
use core::convert::TryInto;
use core::ops::DerefMut;
use pc_keyboard::DecodedKey;

use keyboard::KEYBUFFER;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    println!("Hello RCPU_{}S", 1);

    init();

    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };

    // RCPU memory starts right after the multiboot structure
    let rcpu_mem_start = boot_info.end_address();

    // To find the end, find the memory area containing the start address
    let memory_map_tag = boot_info.memory_map_tag()
                .expect("Memory map tag required");
    let mut rcpu_mem_end: usize = rcpu_mem_start;
    for memory_area in memory_map_tag.memory_areas() {
        if memory_area.start_address() <= rcpu_mem_start.try_into().unwrap() &&
            memory_area.end_address() >= rcpu_mem_start.try_into().unwrap() {
                rcpu_mem_end = memory_area.end_address().try_into().unwrap();
            }
    }

    loop {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut keybuffer = KEYBUFFER.lock();
            for key in keybuffer.deref_mut() {
                println!("{:?}", key);
            }
        });
        x86_64::instructions::hlt();
    }

    // TODO For now, just select the first module
    let mut running_program = rcpu::RCPUProgram::from_module_tag(
        boot_info.module_tags().next().expect("No modules found!"),
        rcpu_mem_start,
        rcpu_mem_end
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

