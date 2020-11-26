use crate::println;
use crate::memory::memcpy;

use multiboot2::ModuleTag;

struct RCPUState {
    ip: u16,
    sp: u16,
    a: i16,
    b: i16,
    c: i16,
    d: i16
}

pub struct RCPUProgram {
    code_start: *mut u16,
    stack_top: *mut u16,
    state: RCPUState,
}

impl RCPUProgram {
    fn read(&self, index: usize) -> u16 {
        if index > 0xffff {
            panic!("Read at index {} out of bounds!", index);
        }

        let ret;
        unsafe {
            ret = *self.code_start.offset(index as isize);
        }
        ret
    }

    fn write(&self, index: usize, value: u16) {
        if index > 0xffff {
            panic!("Write at index {} out of bounds!", index);
        }

        unsafe {
            *self.code_start.offset(index as isize) = value;
        }
    }

    pub fn from_module_tag(tag: &ModuleTag, working_space_start: usize) -> RCPUProgram {
        // Print the name
        println!("Booting {}", tag.name());

        // Copy the program to the working space
        unsafe {
            memcpy(
                working_space_start as *mut u8,
                tag.start_address() as *const u8,
                (tag.end_address() - tag.start_address()) as usize
            );
        }

        // TODO init stack
        RCPUProgram {
            code_start: working_space_start as *mut u16,
            stack_top: 0 as *mut u16,
            state: RCPUState {
                ip: 0,
                sp: 0,
                a: 0,
                b: 0,
                c: 0,
                d: 0
            }
        }
    }
}
