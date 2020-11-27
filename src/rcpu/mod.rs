use crate::println;
use crate::memory::memcpy;
use crate::memory::swap_endianness;

mod operations;

use crate::rcpu::operations::RCPUInstructionType;
use crate::rcpu::operations::RCPUAthOperation;
use crate::rcpu::operations::RCPUAthMode;
use crate::rcpu::operations::RCPUOperation;

use multiboot2::ModuleTag;

// TODO: split up mod.rs (runner) to state.rs
#[derive(Debug,Clone,Copy)]
pub enum RCPURegister {
    A = 0,
    B,
    C,
    D,
    IP,
    SP
}

#[derive(Debug,Clone,Copy)]
struct RCPUState {
    ip: u16,
    sp: u16,
    a: u16,
    b: u16,
    c: u16,
    d: u16
}

#[derive(Debug,Clone,Copy)]
pub struct RCPUProgram {
    pub running: bool,
    code_start: *mut u16,
    stack_top: *mut u16,
    state: RCPUState,
}

impl RCPUProgram {
    fn read(&self, index: u16) -> u16 {
        let ret;
        unsafe {
            ret = *self.code_start.offset(index as isize);
        }
        println!("Read {} as {}", index, ret);
        // TODO: this is kinda ugly
        swap_endianness(ret)
    }

    fn write(&self, index: u16, value: u16) {
        println!("Wrote {} to {}", value, index);
        unsafe {
            *self.code_start.offset(index as isize) = swap_endianness(value);
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
            running: true,
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

    fn set_register(&mut self, register: RCPURegister, value: u16) {
        match register {
            RCPURegister::A => self.state.a = value,
            RCPURegister::B => self.state.b = value,
            RCPURegister::C => self.state.c = value,
            RCPURegister::D => self.state.d = value,
            RCPURegister::IP => self.state.ip = value,
            RCPURegister::SP => self.state.sp = value,
        }
    }

    fn get_register(&self, register: RCPURegister) -> u16 {
        match register {
            RCPURegister::A => self.state.a,
            RCPURegister::B => self.state.b,
            RCPURegister::C => self.state.c,
            RCPURegister::D => self.state.d,
            RCPURegister::IP => self.state.ip,
            RCPURegister::SP => self.state.sp,
        }
    }

    fn inc_register(&mut self, register: RCPURegister) {
        let value = self.get_register(register);
        self.set_register(register, value.wrapping_add(1));
    }

    fn execute(&mut self, operation: RCPUOperation) {
        match operation.instruction_type() {
            RCPUInstructionType::MOV => {
                let value = self.get_register(operation.source_register());
                self.set_register(operation.destination_register(), value);
                self.inc_register(RCPURegister::IP);
            },
            RCPUInstructionType::LDV => {
                let value = operation.value();
                self.set_register(operation.destination_register(), value);
                self.inc_register(RCPURegister::IP);
            },
            RCPUInstructionType::LDA => {
                let value = self.read(operation.value());
                self.set_register(operation.destination_register(), value);
                self.inc_register(RCPURegister::IP);
            },
            RCPUInstructionType::LDM => {
                let value = self.get_register(operation.destination_register());
                self.write(operation.value(), value);
                self.inc_register(RCPURegister::IP);
            },
            RCPUInstructionType::LDR => {
                let memory_address = self.get_register(operation.source_register());
                let value = self.read(memory_address);
                self.set_register(operation.destination_register(), value);
                self.inc_register(RCPURegister::IP);
            },
            RCPUInstructionType::LDP => {
                let value = self.get_register(operation.source_register());
                let memory_address = self.get_register(operation.destination_register());
                self.write(memory_address, value);
                self.inc_register(RCPURegister::IP);
            },
            RCPUInstructionType::ATH => {
                // Load the values
                let src_value = self.get_register(operation.source_register());
                let dest_value = self.get_register(operation.destination_register());

                // Calculate the new value
                let new_value: u16;
                match operation.ath_operation() {
                    RCPUAthOperation::Add => {
                        new_value = src_value.wrapping_add(dest_value);
                    }
                    RCPUAthOperation::Subtract => {
                        new_value = dest_value.wrapping_sub(src_value);
                    }
                    RCPUAthOperation::Multiply => {
                        new_value = dest_value.wrapping_mul(src_value);
                    }
                    RCPUAthOperation::Divide => {
                        new_value = dest_value.wrapping_div(src_value);
                    }
                    RCPUAthOperation::LeftShift => {
                        new_value = src_value.wrapping_shl(
                            operation.ath_shift().into()
                        );
                    }
                    RCPUAthOperation::RightShift => {
                        new_value = src_value.wrapping_shr(
                            operation.ath_shift().into()
                        );
                    }
                    RCPUAthOperation::And => {
                        new_value = src_value & dest_value;
                    }
                    RCPUAthOperation::Or => {
                        new_value = src_value | dest_value;
                    }
                    RCPUAthOperation::Xor => {
                        new_value = src_value ^ dest_value;
                    }
                    RCPUAthOperation::Not => {
                        new_value = !src_value;
                    }
                    RCPUAthOperation::Increment => {
                        new_value = dest_value.wrapping_add(1);
                    }
                    RCPUAthOperation::Decrement => {
                        new_value = dest_value.wrapping_sub(1);
                    }
                }

                // Store the new value
                match operation.ath_mode() {
                    RCPUAthMode::ToSource => self.set_register(
                        operation.source_register(),
                        new_value
                    ),
                    RCPUAthMode::ToDest => self.set_register(
                        operation.destination_register(),
                        new_value
                    ),
                }
                self.inc_register(RCPURegister::IP);
            }
            
            RCPUInstructionType::JLT => {
                if self.get_register(RCPURegister::A) < self.get_register(operation.destination_register()) {
                    self.set_register(
                        RCPURegister::IP,
                        self.get_register(operation.source_register())
                    )
                } else {
                    self.inc_register(RCPURegister::IP);
                }

            }

            RCPUInstructionType::HLT => {
                self.running = false;
            }
            RCPUInstructionType::JMP => {
                let value = operation.value();
                self.set_register(RCPURegister::IP, value);
            },
            RCPUInstructionType::JMR => {
                self.set_register(
                    RCPURegister::IP,
                    self.get_register(operation.source_register())
                );
            }
            _ => {
                unimplemented!(
                    "Unimplemented RCPU instruction {:?}",
                    operation.instruction_type()
                )
            }
        }
    }

    pub fn step(&mut self) {
        // Get the current opcode
        let binary_opcode: u16 = self.read(self.state.ip.into());
        
        // Parse and execute
        let operation = RCPUOperation {
            opcode: binary_opcode
        };
        self.execute(operation);

        println!("New state: {:?}", self.state);
    }
}
