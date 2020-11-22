// Disable standard library
#![no_std]

// Enable the "x86-interrupt" calling convention
#![feature(abi_x86_interrupt)]

extern crate x86_64;
extern crate volatile;
extern crate lazy_static;
extern crate spin;
extern crate pic8259_simple;

pub mod terminal;
pub mod interrupts;
pub mod gdt;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
