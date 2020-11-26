use crate::println;
use crate::memory::memcpy;

use multiboot2::ModuleTag;

pub struct RCPUProgram {
    start: usize,
}

impl RCPUProgram {
    pub fn read(&self, index: usize) -> u16 {
        if index > 0xffff {
            panic!("Read at index {} out of bounds!", index);
        }

        let address = (self.start + index) as *const u16;
        let value;
        unsafe {
            value = *address;
        }
        value
    }

    pub fn write(&self, index: usize, value: u16) {
        if index > 0xffff {
            panic!("Write at index {} out of bounds!", index);
        }

        let address = (self.start + index) as *mut u16;
        unsafe {
            *address = value;
        }
    }

    pub fn from_module_tag(tag: &ModuleTag, working_space_start: usize) -> RCPUProgram {
        // Copy the program to the working space
        unsafe {
            memcpy(
                working_space_start as *mut u8,
                tag.start_address() as *const u8,
                (tag.end_address() - tag.start_address()) as usize
            );
        }

        // Casting to usize as they are u32 according to the spec
        RCPUProgram {
            start: working_space_start
        }
    }
}
