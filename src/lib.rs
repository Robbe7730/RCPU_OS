// Disable standard library
#![no_std]

// Enable the "x86-interrupt" calling convention
#![feature(abi_x86_interrupt)]

extern crate x86_64;
extern crate volatile;
extern crate lazy_static;
extern crate spin;

pub mod terminal;
pub mod interrupts;
pub mod gdt;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
}
