use core::convert::TryInto;
use core::ops::DerefMut;

use crate::print;
use crate::println;
use crate::memory::memcpy;
use crate::memory::swap_endianness;
use crate::keyboard::KEYBUFFER;
use crate::rcpu::operations::RCPUInstructionType;
use crate::rcpu::operations::RCPUAthOperation;
use crate::rcpu::operations::RCPUAthMode;
use crate::rcpu::operations::RCPUOperation;

use multiboot2::ModuleTag;
use multiboot2::MemoryArea;
use pc_keyboard::DecodedKey;
use pc_keyboard::KeyCode;

mod operations;

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
pub enum RCPUSyscall {
    Printf = 0,
    Fgets,
    Getc
}

impl From<u16> for RCPUSyscall {
    fn from(value: u16) -> RCPUSyscall {
        match value {
            0 => RCPUSyscall::Printf,
            1 => RCPUSyscall::Fgets,
            2 => RCPUSyscall::Getc,
            _ => panic!("Invalid syscall number {}", value)
        }
    }
}

#[derive(Debug,Clone,Copy)]
pub struct RCPUProgram {
    pub running: bool,
    ram_start: *mut u16,
    stack_start: *mut u16,
    stack_end: *mut u16,
    state: RCPUState,
}

impl RCPUProgram {
    fn read(&self, index: u16) -> u16 {
        let ret;
        unsafe {
            ret = *self.ram_start.offset(index as isize);
        }
        // println!("Read {} as {}", index, ret);
        // TODO: this is kinda ugly
        swap_endianness(ret)
    }

    fn write(&self, index: u16, value: u16) {
        // println!("Wrote {} to {}", value, index);
        unsafe {
            *self.ram_start.offset(index as isize) = swap_endianness(value);
        }
    }
    
    fn push(&mut self, value: u16) {
        // println!("Pushed {} to the stack", value);
        unsafe {
            let mem_sp = self.stack_start.offset(
                self.get_register(RCPURegister::SP).try_into().unwrap()
            );
            if mem_sp > self.stack_end {
                panic!("Stack overflow");
            }
            *mem_sp = swap_endianness(value)
        }
        self.inc_register(RCPURegister::SP);
    }

    fn pop(&mut self) -> u16 {
        let value;
        if self.get_register(RCPURegister::SP) <= 0 {
            panic!("Stack underflow");
        }
        self.dec_register(RCPURegister::SP);
        unsafe {
            value = *self.stack_start.offset(
                self.get_register(RCPURegister::SP).try_into().unwrap()
            );
        }
        // println!("Popped {} from the stack", swap_endianness(value));
        swap_endianness(value)
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

    fn dec_register(&mut self, register: RCPURegister) {
        let value = self.get_register(register);
        self.set_register(register, value.wrapping_sub(1));
    }

    fn syscall(&mut self) {
        let syscall = RCPUSyscall::from(self.pop());
        match syscall {
            // Pops need to be split up due to compiler problems
            RCPUSyscall::Printf => {
                let fmt = self.pop();
                self.print_string(fmt, true);
            }
            RCPUSyscall::Getc => {
                let stream_num = self.pop();
                let char_read = self.get_character(stream_num);
                self.push(char_read);
            }
            RCPUSyscall::Fgets => {
                let str_ptr = self.pop();
                let size = self.pop();
                let stream_num = self.pop();
                let chars_read = self.get_string(str_ptr, size, stream_num);
                self.push(chars_read);
            }
        }
    }

    fn print_string(&mut self, str_pointer: u16, should_format: bool) {
        let mut curr_char_idx = str_pointer;
        let mut curr_char = self.read(curr_char_idx) as u8 as char; 
        let mut formatting = false;
        while curr_char != '\0' {
            if should_format && formatting {
                match curr_char {
                    'd' => print!("{}", self.pop()),
                    'c' => print!("{}", self.pop() as u8 as char),
                    // This needs to be split up due to compiler problems
                    's' => { let value = self.pop(); self.print_string(value, false); },
                    '%' => print!("%"),
                    _ => panic!("Invalid formatter %{}", curr_char)
                }
                formatting = false;
            } else if should_format && curr_char == '%' {
                formatting = true;
            } else {
                print!("{}", curr_char);
            }
            curr_char_idx += 1;
            curr_char = self.read(curr_char_idx) as u8 as char;
        }
    }

    fn get_character(&mut self, stream_num: u16) -> u16 {
        // TODO: ASCII checks
        if stream_num != 0 {
            panic!("Invalid stream number: {}", stream_num);
        }

        let mut ret: u16 = u16::MAX;
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut keybuffer = KEYBUFFER.lock();
            for key in keybuffer.deref_mut() {
                if let DecodedKey::Unicode(c) = key {
                    ret = c as u16;
                    break;
                }
            }
        });
        return ret as u16;
    }

    fn get_string(&mut self, str_ptr: u16, size: u16, stream_num: u16) -> u16 {
        // TODO: ASCII checks
        if stream_num != 0 {
            panic!("Invalid stream number: {}", stream_num);
        }

        let mut chars_read = 0;
        let mut hit_nullbyte = false;
        while chars_read < size && !hit_nullbyte {
            let mut char_read = false;
            x86_64::instructions::interrupts::without_interrupts(|| {
                let mut keybuffer = KEYBUFFER.lock();
                if let Some(key) = keybuffer.deref_mut().next() {
                    let ret = match key {
                        DecodedKey::Unicode(c) => Some(c),
                        DecodedKey::RawKey(KeyCode::F1) => Some('\0'),
                        _ => None
                    };
                    if let Some(c) = ret {
                        self.write(str_ptr + chars_read, c as u16);
                        char_read = true;
                        hit_nullbyte = c == '\0';
                    }
                }
            });

            if char_read {
                chars_read += 1;
            } else {
                x86_64::instructions::interrupts::enable_and_hlt();
            }
        }

        // Add ending nullbyte
        if !hit_nullbyte {
            self.write(str_ptr + chars_read, 0);
        }
        return chars_read;
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
            RCPUInstructionType::CAL => {
                self.inc_register(RCPURegister::IP);
                self.push(self.get_register(RCPURegister::IP));
                let new_ip = self.get_register(operation.destination_register());
                self.set_register(RCPURegister::IP, new_ip);
            }
            RCPUInstructionType::RET => {
                let new_ip = self.pop();
                self.set_register(RCPURegister::IP, new_ip);
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
            RCPUInstructionType::PSH => {
                let value = self.get_register(operation.source_register());
                self.push(value);
                self.inc_register(RCPURegister::IP);
            }
            RCPUInstructionType::POP => {
                let value = self.pop();
                self.set_register(operation.destination_register(), value);
                self.inc_register(RCPURegister::IP);
            }
            RCPUInstructionType::SYS => {
                self.syscall();
                self.inc_register(RCPURegister::IP);
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

        // println!("New state: {:?}", self.state);
    }

    pub fn from_module_tag(tag: &ModuleTag, rcpu_mem_start: usize, rcpu_mem_end: usize) -> RCPUProgram {
        // Print the name
        println!("Booting {}", tag.name());

        // Set the start and end pointers
        let ram_start = rcpu_mem_start as *mut u16;
        let stack_start = (rcpu_mem_start + 65536) as *mut u16;
        let stack_end = rcpu_mem_end as *mut u16;

        // Copy the program to the RAM
        unsafe {
            memcpy(
                ram_start as *mut u8,
                tag.start_address() as *const u8,
                (tag.end_address() - tag.start_address()) as usize
            );
        }

        RCPUProgram {
            running: true,
            ram_start: ram_start,
            stack_start: stack_start,
            stack_end: stack_end,
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
