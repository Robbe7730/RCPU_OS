// Bitmap for the different fields in the opcode
const OPCODE_BITMAP: u16        = 0b0000000000001111;
const SOURCE_BITMAP: u16        = 0b0000000000110000;
const DESTINATION_BITMAP: u16   = 0b0000000011000000;
const ATH_OPERATION_BITMAP: u16 = 0b0000111100000000;
const ATH_MODE_BITMAP: u16      = 0b0001000000000000;
const ATH_SHIFT_BITMAP: u16     = 0b1110000000000000;
const VALUE_BITMAP: u16     = 0b1111111111000000;

// How much to shift after applying the bitmap
const OPCODE_SHIFT: u8          = 0; // Actually unneccesary, but kept for consistency
const SOURCE_SHIFT: u8          = 4;
const DESTINATION_SHIFT: u8     = 6;
const ATH_OPERATION_SHIFT: u8   = 8;
const ATH_MODE_SHIFT: u8        = 12;
const ATH_SHIFT_SHIFT: u8       = 13;
const VALUE_SHIFT: u8           = 6;

use crate::rcpu::RCPURegister;

#[derive(Debug)]
pub enum RCPUInstructionType {
    MOV = 0,
    LDV,
    LDA,
    LDM,
    LDR,
    LDP,
    ATH,
    CAL,
    RET,
    JLT,
    PSH,
    POP,
    SYS,
    HLT,
    JMP,
    JMR
}

#[derive(Debug)]
pub enum RCPUAthOperation {
    Add = 0,
    Subtract,
    Multiply,
    Divide,
    LeftShift,
    RightShift,
    And,
    Or,
    Xor,
    Not,
    Increment,
    Decrement,
}

#[derive(Debug)]
pub enum RCPUAthMode {
    ToDest = 0,
    ToSource
}

impl RCPURegister {
    pub fn from_u16(number: u16) -> RCPURegister {
        match number {
            0 => RCPURegister::A,
            1 => RCPURegister::B,
            2 => RCPURegister::C,
            3 => RCPURegister::D,
            _ => panic!("Invalid register number {}", number)
        }
    }
}

#[derive(Debug)]
pub struct RCPUOperation {
    pub opcode: u16
}

impl RCPUOperation {
    pub fn instruction_type(&self) -> RCPUInstructionType {
        match (self.opcode & OPCODE_BITMAP) >> OPCODE_SHIFT {
            0 => RCPUInstructionType::MOV,
            1 => RCPUInstructionType::LDV,
            2 => RCPUInstructionType::LDA,
            3 => RCPUInstructionType::LDM,
            4 => RCPUInstructionType::LDR,
            5 => RCPUInstructionType::LDP,
            6 => RCPUInstructionType::ATH,
            7 => RCPUInstructionType::CAL,
            8 => RCPUInstructionType::RET,
            9 => RCPUInstructionType::JLT,
            10 => RCPUInstructionType::PSH,
            11 => RCPUInstructionType::POP,
            12 => RCPUInstructionType::SYS,
            13 => RCPUInstructionType::HLT,
            14 => RCPUInstructionType::JMP,
            15 => RCPUInstructionType::JMR,
            _ => panic!("Unreachable statement"),
        }
    }
    
    pub fn source_register(&self) -> RCPURegister {
        RCPURegister::from_u16((self.opcode & SOURCE_BITMAP) >> SOURCE_SHIFT)
    }

    pub fn destination_register(&self) -> RCPURegister {
        RCPURegister::from_u16((self.opcode & DESTINATION_BITMAP) >> DESTINATION_SHIFT)
    }

    pub fn value(&self) -> u16 {
        (self.opcode & VALUE_BITMAP) >> VALUE_SHIFT
    }

    pub fn ath_operation(&self) -> RCPUAthOperation {
        let bin_operation = (self.opcode & ATH_OPERATION_BITMAP) >> ATH_OPERATION_SHIFT;
        match bin_operation {
            0 => RCPUAthOperation::Add,
            1 => RCPUAthOperation::Subtract,
            2 => RCPUAthOperation::Multiply,
            3 => RCPUAthOperation::Divide,
            4 => RCPUAthOperation::LeftShift,
            5 => RCPUAthOperation::RightShift,
            6 => RCPUAthOperation::And,
            7 => RCPUAthOperation::Or,
            8 => RCPUAthOperation::Xor,
            9 => RCPUAthOperation::Not,
            10 => RCPUAthOperation::Increment,
            11 => RCPUAthOperation::Decrement,
            _ => panic!("Invalid ATH operation {:b}", bin_operation),
        }
    }

    pub fn ath_mode(&self) -> RCPUAthMode {
        match (self.opcode & ATH_MODE_BITMAP) >> ATH_MODE_SHIFT {
            0 => RCPUAthMode::ToDest,
            1 => RCPUAthMode::ToSource,
            _ => panic!("Unreachable statement"),
        }
    }

    pub fn ath_shift(&self) -> u16 {
        (self.opcode & ATH_SHIFT_BITMAP) >> ATH_SHIFT_SHIFT
    }
}
