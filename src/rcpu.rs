use crate::println;

use multiboot2::ModuleTag;

pub struct RCPUProgram {
    start: usize,
    end: usize,
}

impl RCPUProgram {
    pub fn read(&self, index: usize) -> u16 {
        if self.start + index > self.end {
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
        if self.start + index > self.end {
            panic!("Write at index {} out of bounds!", index);
        }

        let address = (self.start + index) as *mut u16;
        unsafe {
            *address = value;
        }
    }

    pub fn from_module_tag(tag: &ModuleTag) -> RCPUProgram {
        // Casting to usize as they are u32 according to the spec
        RCPUProgram {
            start: tag.start_address() as usize,
            end: tag.end_address() as usize,
        }
    }
}
