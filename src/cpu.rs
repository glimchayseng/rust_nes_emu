#[allow(unused_imports)]
use std::{collections::{btree_map::Values, HashMap}, hash::Hash};
use crate::opcodes;

bitflags! {
    /// # Status Register (P) http://wiki.nesdev.com/w/index.php/Status_flags
    ///
    ///  7 6 5 4 3 2 1 0
    ///  N V _ B D I Z C
    ///  | |   | | | | +--- Carry Flag
    ///  | |   | | | +----- Zero Flag
    ///  | |   | | +------- Interrupt Disable
    ///  | |   | +--------- Decimal Mode (not used on NES)
    ///  | |   +----------- Break Command
    ///  | +--------------- Overflow Flag
    ///  +----------------- Negative Flag
    ///
    pub struct CpuFlags: u8 {
        const CARRY             = 0b00000001;
        const ZERO              = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100;
        const DECIMAL_MODE      = 0b00001000;
        const BREAK             = 0b00010000;
        const BREAK2            = 0b00100000;
        const OVERFLOW          = 0b01000000;
        const NEGATIV           = 0b10000000;
    }
}

#[allow(dead_code)]
const STACK: u16 = 0x0100;
const STACK_RESET: u8 = 0xfd;

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: CpuFlags,
    pub stack_pointer:u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF]
 }

 #[derive(Debug)]
 #[allow(non_camel_case_types)]
 #[allow(dead_code)]
 pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
 }
  
 pub trait Mem {
    fn mem_read(&self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}

impl Mem for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
}

impl CPU {
pub fn new() -> Self {
    CPU {
        register_a: 0,
        register_x: 0,
        register_y: 0,
        stack_pointer: STACK_RESET,
        program_counter: 0,
        status: CpuFlags::from_bits_truncate(0b100100),
        memory: [0; 0xFFFF],
    }
}

fn set_register_a(&mut self, value: u8) {
    self.register_a = value;
    self.update_zero_and_negative_flags(self.register_a);
}

fn add_to_register_a(&mut self, value: u8) {
    let sum = self.register_a as u16
        + value as u16
        + (if self.status.contains(CpuFlags::CARRY) {
            1
        } else {
            0
        }) as u16;

    let carry = sum > 0xFF;

    if carry {
        self.status.insert(CpuFlags::CARRY);
    } else {
        self.status.remove(CpuFlags::CARRY);
    }

    let result = sum as u8;

    if (value ^ result) & (result ^ self.register_a) & 0x80 != 0 {
        self.status.insert(CpuFlags::OVERFLOW);
    } else {
        self.status.remove(CpuFlags::OVERFLOW);
    }

    self.set_register_a(result);
}

fn adc(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.add_to_register_a(value);
}

fn sbc(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.add_to_register_a(((value as i8).wrapping_neg().wrapping_sub(1)) as u8);
}

fn and(&mut self, mode:&AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.set_register_a(value & self.register_a);
}

fn asl_accumulator(&mut self) {
    let mut value = self.register_a;
    if value >> 7 == 1 {
        self.set_carry_flag();
    } else {
        self.clear_carry_flag();
    }
    value = value << 1;
    self.set_register_a(value)
}

fn asl(&mut self, mode: &AddressingMode) -> u8 {
    let addr = self.get_operand_address(mode);
    let mut value = self.mem_read(addr);

    if value >> 7 == 1 {
        self.set_carry_flag();
    } else {
        self.clear_carry_flag();
    }

    value = value << 1;
    self.mem_write(addr, value);
    self.update_zero_and_negative_flags(value);
    value
}

fn branch(&mut self, condition: bool) {
    if condition {
        let jump: i8 = self.mem_read(self.program_counter) as i8;
        let jump_addr = self
            .program_counter
            .wrapping_add(1)
            .wrapping_add(jump as u16);

        self.program_counter = jump_addr;
    }
}

fn bit(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);
    let and = self.register_a & value;
    if and == 0 {
        self.status.insert(CpuFlags::ZERO);
    } else {
        self.status.remove(CpuFlags::ZERO);
    }

    self.status.set(CpuFlags::NEGATIV, value & 0b10000000 > 0);
    self.status.set(CpuFlags::OVERFLOW, value & 0b01000000 > 0);
}

fn compare(&mut self, mode: &AddressingMode, compare_with: u8) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    if value <= compare_with {
        self.status.insert(CpuFlags::CARRY);
    } else {
        self.status.remove(CpuFlags::CARRY);
    }

    self.update_zero_and_negative_flags(compare_with.wrapping_sub(value));
}

fn dec(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);
    
    let result = value.wrapping_sub(1);

    self.mem_write(addr, result);
    self.update_zero_and_negative_flags(result);
}

fn dex(&mut self) {
    self.register_x = self.register_x.wrapping_sub(1);
    self.update_zero_and_negative_flags(self.register_x);
}

fn dey(&mut self) {
    self.register_y = self.register_y.wrapping_sub(1);
    self.update_zero_and_negative_flags(self.register_y);
}

fn eor(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.set_register_a(value ^ self.register_a);
}

fn inc(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let mut value = self.mem_read(addr);

    value = value.wrapping_add(1);

    self.mem_write(addr, value);
    self.update_zero_and_negative_flags(value);
}

fn inx(&mut self) {
    self.register_x = self.register_x.wrapping_add(1);
    self.update_zero_and_negative_flags(self.register_x);
}

fn iny(&mut self) {
    self.register_y = self.register_y.wrapping_add(1);
    self.update_zero_and_negative_flags(self.register_y);
}

fn lda(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);
    
    self.set_register_a(value);
}

fn ldx(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);
    
    self.register_x = value;
    self.update_zero_and_negative_flags(self.register_x);
}

fn ldy(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);
    
    self.register_y = value;
    self.update_zero_and_negative_flags(self.register_y);
}

fn lsr_accumulator(&mut self) {
    let mut value = self.register_a;

    if value & 1 == 1 {
        self.set_carry_flag();
    } else {
        self.clear_carry_flag();
    }

    value = value >> 1;
    self.set_register_a(value)
}

fn lsr(&mut self, mode: &AddressingMode) -> u8 {
    let addr = self.get_operand_address(mode);
    let mut value = self.mem_read(addr);

    if value & 1 == 1 {
        self.set_carry_flag();
    } else {
        self.clear_carry_flag();
    }

    value = value >> 1;
    self.mem_write(addr, value);
    self.update_zero_and_negative_flags(value);
    value
}

fn ora(&mut self, mode:&AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.set_register_a(self.register_a | value);
}

fn php(&mut self) {
    let mut flags = self.status.clone();
    flags.insert(CpuFlags::BREAK);
    flags.insert(CpuFlags::BREAK2);
    self.stack_push(flags.bits());
}

fn pla(&mut self) {
    let data = self.stack_pop();
    self.set_register_a(data);
}

fn plp(&mut self) {
    self.status.bits = self.stack_pop();
    self.status.remove(CpuFlags::BREAK);
    self.status.insert(CpuFlags::BREAK2);
}

fn rol(&mut self, mode: &AddressingMode) -> u8 {
    let addr = self.get_operand_address(mode);
    let mut value = self.mem_read(addr);
    let old_carry = self.status.contains(CpuFlags::CARRY);

    if value >> 7 == 1 {
        self.set_carry_flag();
    } else {
        self.clear_carry_flag();
    }
    value = value << 1;
    if old_carry {
        value = value | 1;
    }
    self.mem_write(addr, value);
    self.update_zero_and_negative_flags(value);
    value
}

fn rol_accumulator(&mut self) {
    let mut value = self.register_a;
    let old_carry = self.status.contains(CpuFlags::CARRY);

    if value >> 7 == 1 {
        self.set_carry_flag();
    } else {
        self.clear_carry_flag();
    }
    value = value << 1;
    if old_carry {
        value = value | 1;
    }
    self.set_register_a(value);
}

fn ror(&mut self, mode: &AddressingMode) -> u8 {
    let addr = self.get_operand_address(mode);
    let mut value = self.mem_read(addr);
    let old_carry = self.status.contains(CpuFlags::CARRY);

    if value & 1 == 1 {
        self.set_carry_flag();
    } else {
        self.clear_carry_flag();
    }
    value = value >> 1;
    if old_carry {
        value = value | 0b10000000;
    }
    self.mem_write(addr, value);
    self.update_zero_and_negative_flags(value);
    value
}

fn ror_accumulator(&mut self) {
    let mut value = self.register_a;
    let old_carry = self.status.contains(CpuFlags::CARRY);

    if value & 1 == 1 {
        self.set_carry_flag();
    } else {
        self.clear_carry_flag();
    }
    value = value >> 1;
    if old_carry {
        value = value | 0b10000000;
    }
    self.set_register_a(value);
}

fn sta(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    self.mem_write(addr, self.register_a);
}

fn stx(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    self.mem_write(addr, self.register_x);
}

fn sty(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    self.mem_write(addr, self.register_y);
}

fn tax(&mut self) {
    self.register_x = self.register_a;
    self.update_zero_and_negative_flags(self.register_x);
}

fn tay(&mut self) {
    self.register_y = self.register_a;
    self.update_zero_and_negative_flags(self.register_y);
}

fn tsx(&mut self) {
    self.register_x = self.stack_pointer;
    self.update_zero_and_negative_flags(self.register_x);
}

fn txa(&mut self) {
    self.register_a = self.register_x;
    self.update_zero_and_negative_flags(self.register_a);
}

fn txs(&mut self) {
    self.stack_pointer = self.register_x;
}

fn tya(&mut self) {
    self.register_a = self.register_y;
    self.update_zero_and_negative_flags(self.register_a);
}

fn update_zero_and_negative_flags(&mut self, result: u8) {
    if result == 0 {
        self.status.insert(CpuFlags::ZERO);
    } else {
        self.status.remove(CpuFlags::ZERO);
    }

    if result & 0b1000_0000 != 0 {
        self.status.insert(CpuFlags::NEGATIV);
    } else {
        self.status.remove(CpuFlags::NEGATIV);
    }
}

fn set_carry_flag(&mut self) {
    self.status.insert(CpuFlags::CARRY);
}

fn clear_carry_flag(&mut self) {
    self.status.remove(CpuFlags::CARRY);
}

pub fn reset(&mut self) {
    self.register_a = 0;
    self.register_x = 0;
    self.register_y = 0;
    self.stack_pointer = STACK_RESET;
    self.status = CpuFlags::from_bits_truncate(0b100100);

    self.program_counter = self.mem_read_u16(0xFFFC);
}

// pub fn load(&mut self, program: Vec<u8>) {
//     self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
//     self.mem_write_u16(0xFFFC, 0x8000);
// }

pub fn load(&mut self, program: Vec<u8>) {
    self.memory[0x0600..(0x0600 + program.len())].copy_from_slice(&program[..]);
    self.mem_write_u16(0xFFFC, 0x0600);
}

pub fn load_and_run(&mut self, program: Vec<u8>) {
    self.load(program);
    self.reset();
    self.run()
}

fn stack_push(&mut self, data: u8) {
    self.mem_write((STACK as u16) + self.stack_pointer as u16, data);
    self.stack_pointer = self.stack_pointer.wrapping_sub(1)
}

fn stack_push_u16(&mut self, data: u16) {
    let hi = (data >> 8) as u8;
    let lo = (data & 0xff) as u8;
    self.stack_push(hi);
    self.stack_push(lo);
}

fn stack_pop(&mut self) -> u8 {
    self.stack_pointer = self.stack_pointer.wrapping_add(1);
    self.mem_read((STACK as u16) + self.stack_pointer as u16)
}

fn stack_pop_u16(&mut self) -> u16 {
    let lo = self.stack_pop() as u16;
    let hi = self.stack_pop() as u16;

    hi << 8 | lo
}

pub fn run(&mut self) {
    self.run_with_callback(|_| {});
}

pub fn run_with_callback<F>(&mut self, mut callback: F) where F: FnMut(&mut CPU), {
    let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;

    loop {
        let code = self.mem_read(self.program_counter);
        self.program_counter += 1;
        let program_counter_state = self.program_counter;

        let opcode = opcodes.get(&code).expect(&format!("OpCode {:x} is not recognized", code));

        match code {
            // ADC opcodes
            0x69 | 0x65 | 0x75 | 0x6d | 0x7d | 0x79 | 0x61 | 0x71 => {
                self.adc(&opcode.adr_mode);
            }

            // SBC opcodes
            0xe9 | 0xe5 | 0xf5 | 0xed | 0xfd | 0xf9 | 0xe1 | 0xf1 => {
                self.sbc(&opcode.adr_mode);
            }

            // AND opcodes
            0x29 | 0x25 | 0x35 | 0x2d | 0x3d | 0x39 | 0x21 | 0x31 => {
                self.and(&opcode.adr_mode);
            }

            // ASL opcodes
            0x0a => self.asl_accumulator(),

            0x06 | 0x16 | 0x0e | 0x1e => {
                self.asl(&opcode.adr_mode);
            }

            // BCC
            0x90 => self.branch(!self.status.contains(CpuFlags::CARRY)),

            // BCS
            0xb0 => self.branch(self.status.contains(CpuFlags::CARRY)),

            // BEQ
            0xf0 => self.branch(self.status.contains(CpuFlags::ZERO)),

            // BNE
            0xd0 => self.branch(!self.status.contains(CpuFlags::ZERO)),

            // BMI
            0x30 => self.branch(self.status.contains(CpuFlags::NEGATIV)),

            // BPL
            0x10 => self.branch(!self.status.contains(CpuFlags::NEGATIV)),

            // BVS
            0x70 => self.branch(self.status.contains(CpuFlags::OVERFLOW)),

            // BVC
            0x50 => self.branch(!self.status.contains(CpuFlags::OVERFLOW)),

            // BIT opcodes
            0x24 | 0x2C => {
                self.bit(&opcode.adr_mode);
            }

            // CLD
            0xd8 => self.status.remove(CpuFlags::DECIMAL_MODE),

            // CLI
            0x58 => self.status.remove(CpuFlags::INTERRUPT_DISABLE),

            // CLV
            0xb8 => self.status.remove(CpuFlags::OVERFLOW),

            // CLC
            0x18 => self.clear_carry_flag(),

            // SEC
            0x38 => self.set_carry_flag(),

            // SEI
            0x78 => self.status.insert(CpuFlags::INTERRUPT_DISABLE),

            // SED
            0xf8 => self.status.insert(CpuFlags::DECIMAL_MODE),

            // CMP opcodes
            0xc9 | 0xc5 | 0xd5 | 0xcd | 0xdd | 0xd9 | 0xc1 | 0xd1 => {
                self.compare(&opcode.adr_mode, self.register_a);
            }

            // CPX opcodes
            0xe0 | 0xe4 | 0xec => {
                self.compare(&opcode.adr_mode, self.register_x);
            }

            // CPY opcodes
            0xc0 | 0xc4 | 0xcc => {
                self.compare(&opcode.adr_mode, self.register_y);
            }

            // DEC
            0xc6 | 0xd6| 0xce | 0xde => {
                self.dec(&opcode.adr_mode);
            }

            // DEX
            0xCA => self.dex(),

            // DEY
            0x88 => self.dey(),

            // EOR
            0x49 | 0x45 | 0x55 | 0x4d | 0x5d | 0x59 | 0x41 | 0x51 => {
                self.eor(&opcode.adr_mode);
            }

            //INC Opcodes
            0xe6 | 0xf6 | 0xee | 0xfe => {
                self.inc(&opcode.adr_mode);
            }

            // INX
            0xE8 => self.inx(),

            // INY
            0xc8 => self.iny(),

            // JMP Absolute
            0x4c => {
                let mem_address = self.mem_read_u16(self.program_counter);
                self.program_counter = mem_address;
            }

            // JMP Indirect 
            0x6c => {
                let mem_address = self.mem_read_u16(self.program_counter);

                let indirect_ref = if mem_address & 0x00FF == 0x00FF {
                    let lo = self.mem_read(mem_address);
                    let hi = self.mem_read(mem_address & 0xFF00);
                    (hi as u16) << 8 | (lo as u16)
                } else {
                    self.mem_read_u16(mem_address)
                };

                self.program_counter = indirect_ref;
            }

            // JSR
            0x20 => {
                self.stack_push_u16(self.program_counter + 2 - 1);
                let target_address = self.mem_read_u16(self.program_counter);
                self.program_counter = target_address;
            }

            // LDA opcodes
            0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => {
                self.lda(&opcode.adr_mode);
            }

            // LDX 
            0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe => {
                self.ldx(&opcode.adr_mode);
            }

            // LDY 
            0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc => {
                self.ldy(&opcode.adr_mode);
            }

            // LSR 
            0x4a => self.lsr_accumulator(),
            
            0x46 | 0x56 | 0x4e | 0x5e => {
                self.lsr(&opcode.adr_mode);
            }

            // NOP
            0xEA => {} // Do nothing :D

            // ORA
            0x09 | 0x05 | 0x15 | 0x0d | 0x1d | 0x19 | 0x01 | 0x11 => {
                self.ora(&opcode.adr_mode);
            }

            // PHA
            0x48 => self.stack_push(self.register_a),

            // PHP
            0x08 => self.php(),

            // PLA
            0x68 => self.pla(),

            // PLP
            0x28 => self.plp(),

            // ROL
            0x2a => self.rol_accumulator(),

            0x26 | 0x36 | 0x2e | 0x3e => {
                self.rol(&opcode.adr_mode);
            }

            // ROR
            0x6a => self.ror_accumulator(),

            0x66 | 0x76 | 0x6e | 0x7e => {
                self.ror(&opcode.adr_mode);
            }

            // RTI
            0x40 => {
                self.status.bits = self.stack_pop();
                self.status.remove(CpuFlags::BREAK);
                self.status.insert(CpuFlags::BREAK2);

                self.program_counter = self.stack_pop_u16();
            }

            // RTS
            0x60 => self.program_counter = self.stack_pop_u16() + 1,

            // STA opcodes
            0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => {
                self.sta(&opcode.adr_mode);
            }

            // STX opcodes
            0x86 | 0x96 | 0x8e => {
                self.stx(&opcode.adr_mode);
            }

            // STY opcodes
            0x84 | 0x94 | 0x8c => {
                self.sty(&opcode.adr_mode);
            }
            
            // TAX
            0xaa => self.tax(),

            // TAY
            0xa8 => self.tay(),

            // TSX
            0xba => self.tsx(),

            // TXA 
            0x8a => self.txa(),

            // TXS
            0x9a => self.txs(),

            // TYA
            0x98 => self.tya(),

            // BRK
            0x00 => return,

            _=> println!("Unexpected Value! This shouldnt happen!"),
        }

        if program_counter_state == self.program_counter {
            self.program_counter += (opcode.bytes - 1) as u16;
        }

        callback(self);
    }
}

fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {

    match mode {
        AddressingMode::Immediate => self.program_counter,

        AddressingMode::ZeroPage  => self.mem_read(self.program_counter) as u16,
        
        AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
        
        AddressingMode::ZeroPage_X => {
            let pos = self.mem_read(self.program_counter);
            let addr = pos.wrapping_add(self.register_x) as u16;
            addr
        }
        AddressingMode::ZeroPage_Y => {
            let pos = self.mem_read(self.program_counter);
            let addr = pos.wrapping_add(self.register_y) as u16;
            addr
        }

        AddressingMode::Absolute_X => {
            let base = self.mem_read_u16(self.program_counter);
            let addr = base.wrapping_add(self.register_x as u16);
            addr
        }
        AddressingMode::Absolute_Y => {
            let base = self.mem_read_u16(self.program_counter);
            let addr = base.wrapping_add(self.register_y as u16);
            addr
        }

        AddressingMode::Indirect_X => {
            let base = self.mem_read(self.program_counter);

            let ptr: u8 = (base as u8).wrapping_add(self.register_x);
            let lo = self.mem_read(ptr as u16);
            let hi = self.mem_read(ptr.wrapping_add(1) as u16);
            (hi as u16) << 8 | (lo as u16)
        }
        AddressingMode::Indirect_Y => {
            let base = self.mem_read(self.program_counter);

            let lo = self.mem_read(base as u16);
            let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
            let deref_base = (hi as u16) << 8 | (lo as u16);
            let deref = deref_base.wrapping_add(self.register_y as u16);
            deref
        }
        
        AddressingMode::NoneAddressing => {
            panic!("mode {:?} is not supported", mode);
        }
    }
}

}

 #[cfg(test)]
 #[path = "cpu_tests.rs"]
 mod cpu_tests;