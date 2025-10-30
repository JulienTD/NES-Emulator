use phf::phf_map;

#[derive(Debug)]
pub(crate) struct CPU {
    // More info about the 6502 registers can be found here:
    // https://www.nesdev.org/obelisk-6502-guide/registers.html

    // The program counter is a 16 bit register that holds the memory address of the next instruction to be executed.
    // The value of program counter is modified automatically as instructions are executed.
    pub program_counter: u16,

    // The stack pointer is an 8 bit register and holds the low 8 bits of the next free location
    // on the stack. The location of the stack is fixed and cannot be moved.
    // Memory space [0x0100 .. 0x1FF]
    pub stack_pointer: u8,

    // The accumulator is an 8 bit register used for arithmetic and logical operations.
    pub accumulator: u8,

    // The 8 bit index register is most commonly used to hold counters or offsets for accessing memory.
    pub x_register: u8,

    // The Y register is similar to the X register in that it is available for holding counter or offsets memory access
    pub y_register: u8,

    // As instructions are executed a set of processor flags are set or clear to record the results of the operation. 
    // Each bit in the status register represents a different flag:
    // Bit 7: Negative Flag (N)
    // Bit 6: Overflow Flag (V)
    // Bit 5: Unused (U) (always set to 1)
    // Bit 4: Break Command (B)
    // Bit 3: Decimal Mode Flag (D)
    // Bit 2: Interrupt Disable (I)
    // Bit 1: Zero Flag (Z)
    // Bit 0: Carry Flag (C)
    pub status_register: u8,

    // The 6502 has a 16 bit address bus, which means it can address up to 64KB of memory.
    // This memory is typically divided into several regions, including RAM, ROM, and memory-mapped I/O.
    // Memory map:
    // 0x0000 - 0x1FFF: RAM (mirrored every 0x0800 bytes)
    // 0x2000 - 0x3FFF: PPU Registers (mirrored every 8 bytes)
    // 0x4000 - 0x401F: APU and I/O Registers
    // 0x4020 - 0x5FFF: Expansion ROM
    // 0x6000 - 0x7FFF: Save RAM
    // 0x8000 - 0xFFFF: PRG ROM
    memory: [u8; 0xFFFF],
    // Global cycle counter (counts CPU cycles executed)
    pub cycles: u64,
}

// Each flag corresponds to a bit in the status register
// Values are the bit positions
#[derive(Debug, Clone, Copy)]
pub(crate) enum StatusFlag {
    Carry = 0,
    Zero = 1,
    InterruptDisable = 2,
    DecimalMode = 3,
    BreakCommand = 4,
    Unused = 5,
    Overflow = 6,
    Negative = 7,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum AddressingMode {
    Absolute,    // e.g. LDA $1234
    AbsoluteX,   // e.g. LDA $1234,X
    AbsoluteY,   // e.g. LDA $1234,Y
    Accumulator, // e.g. ASL A
    Immediate,   // e.g. LDA #$10
    Implicit,    // e.g. CLC, INX (no operand)
    Indirect,    // e.g. JMP ($1234)
    IndirectX,   // e.g. LDA ($10,X)
    IndirectY,   // e.g. LDA ($10),Y
    Relative,    // e.g. BEQ +5
    ZeroPage,    // e.g. LDA $10
    ZeroPageX,   // e.g. LDA $10,X
    ZeroPageY,   // e.g. LDX $10,Y
}

pub fn new_cpu() -> CPU {
    CPU {
        program_counter: 0x0000,
        stack_pointer: 0xFF,
        accumulator: 0x00,
        x_register: 0x00,
        y_register: 0x00,
        status_register: 0x00,
        memory: [0; 0xFFFF],
        cycles: 0,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Operand {
    opcode: u8,
    name: &'static str,
    handler: fn(&mut CPU, u8) -> u8,
    addressing_mode: AddressingMode,
    bytes: u8,
    cycles: u8,
}

#[allow(dead_code)]
impl CPU {
    // Addresses for memory regions.
    const RAM_BASE_ADDRESS: u16 = 0x0000;
    const IO_BASE_ADDRESS: u16 = 0x2000;
    const EXP_ROM_BASE_ADDRESS: u16 = 0x4020;
    const SAVE_RAM_BASE_ADDRESS: u16 = 0x6000;
    const PRG_ROM_BASE_ADDRESS: u16 = 0x8000;
    const RESET_VECTOR_ADDRESS: u16 = 0xFFFC;

    // List of all opcodes and their corresponding Operand definitions.
    const OPERAND_MAP: phf::Map<u8, Operand> = phf_map! {
        // ADC Instructions
        0x69 => Operand { opcode: 0x69, name: "ADC", handler: CPU::handleADC, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
        0x65 => Operand { opcode: 0x65, name: "ADC", handler: CPU::handleADC, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
        0x75 => Operand { opcode: 0x75, name: "ADC", handler: CPU::handleADC, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
        0x6D => Operand { opcode: 0x6D, name: "ADC", handler: CPU::handleADC, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
        0x7D => Operand { opcode: 0x7D, name: "ADC", handler: CPU::handleADC, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 4 /* +1 if page crossed */ },
        0x79 => Operand { opcode: 0x79, name: "ADC", handler: CPU::handleADC, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 4 /* +1 if page crossed */ }, // TODO
        0x61 => Operand { opcode: 0x61, name: "ADC", handler: CPU::handleADC, addressing_mode: AddressingMode::IndirectX, bytes: 2, cycles: 6 },
        0x71 => Operand { opcode: 0x71, name: "ADC", handler: CPU::handleADC, addressing_mode: AddressingMode::IndirectY, bytes: 2, cycles: 5 /* +1 if page crossed */ },

        // AND Instructions
        0x29 => Operand { opcode: 0x29, name: "AND", handler: CPU::handleAND, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
        0x25 => Operand { opcode: 0x25, name: "AND", handler: CPU::handleAND, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
        0x35 => Operand { opcode: 0x35, name: "AND", handler: CPU::handleAND, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
        0x2D => Operand { opcode: 0x2D, name: "AND", handler: CPU::handleAND, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
        0x3D => Operand { opcode: 0x3D, name: "AND", handler: CPU::handleAND, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 4 /* +1 if page crossed */ }, // TODO
        0x39 => Operand { opcode: 0x39, name: "AND", handler: CPU::handleAND, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 4 /* +1 if page crossed */ },
        0x21 => Operand { opcode: 0x21, name: "AND", handler: CPU::handleAND, addressing_mode: AddressingMode::IndirectX, bytes: 2, cycles: 6 },
        0x31 => Operand { opcode: 0x31, name: "AND", handler: CPU::handleAND, addressing_mode: AddressingMode::IndirectY, bytes: 2, cycles: 5 /* +1 if page crossed */ },

        // ASL Instructions
        0x0A => Operand { opcode: 0x0A, name: "ASL", handler: CPU::handleASL, addressing_mode: AddressingMode::Accumulator, bytes: 1, cycles: 2 },
        0x06 => Operand { opcode: 0x06, name: "ASL", handler: CPU::handleASL, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 5 },
        0x16 => Operand { opcode: 0x16, name: "ASL", handler: CPU::handleASL, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 6 },
        0x0E => Operand { opcode: 0x0E, name: "ASL", handler: CPU::handleASL, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 6 },
        0x1E => Operand { opcode: 0x1E, name: "ASL", handler: CPU::handleASL, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 7 },

        // BCC Instructions
        0x90 => Operand { opcode: 0x90, name: "BCC", handler: CPU::handleBCC, addressing_mode: AddressingMode::Relative, bytes: 2, cycles: 2 /* +1 if branch succeeds or +2 if to a new page */ },

        // BCS Instructions
        0xB0 => Operand { opcode: 0xB0, name: "BCS", handler: CPU::handleBCS, addressing_mode: AddressingMode::Relative, bytes: 2, cycles: 2 /* +1 if branch succeeds or +2 if to a new page */ },

        // BEQ Instructions
        0xF0 => Operand { opcode: 0xF0, name: "BEQ", handler: CPU::handleBEQ, addressing_mode: AddressingMode::Relative, bytes: 2, cycles: 2 /* +1 if branch succeeds or +2 if to a new page */ },

        // BIT Instructions
        0x24 => Operand { opcode: 0x24, name: "BIT", handler: CPU::handleBit, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
        0x2C => Operand { opcode: 0x2C, name: "BIT", handler: CPU::handleBit, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },

        // BMI Instructions
        0x30 => Operand { opcode: 0x30, name: "BMI", handler: CPU::handleBMI, addressing_mode: AddressingMode::Relative, bytes: 2, cycles: 2 /* +1 if branch succeeds or +2 if to a new page */  },

        // BNE Instructions
        0xD0 => Operand { opcode: 0xD0, name: "BNE", handler: CPU::handleBNE, addressing_mode: AddressingMode::Relative, bytes: 2, cycles: 2 /* +1 if branch succeeds or +2 if to a new page */  },

        // BPL Instructions
        0x10 => Operand { opcode: 0x10, name: "BPL", handler: CPU::handleBPL, addressing_mode: AddressingMode::Relative, bytes: 2, cycles: 2 /* +1 if branch succeeds or +2 if to a new page */  },

        // BRK Instructions
        0x00 => Operand { opcode: 0x00, name: "BRK", handler: CPU::handleBRK, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 7 },

        // BVC Instructions
        0x50 => Operand { opcode: 0x50, name: "BVC", handler: CPU::handleBVC, addressing_mode: AddressingMode::Relative, bytes: 2, cycles: 2 /* +1 if branch succeeds or +2 if to a new page */  },

        // BVS Instructions
        0x70 => Operand { opcode: 0x70, name: "BVS", handler: CPU::handleBVS, addressing_mode: AddressingMode::Relative, bytes: 2, cycles: 2 /* +1 if branch succeeds or +2 if to a new page */  },

        // CLC Instructions
        0x18 => Operand { opcode: 0x18, name: "CLC", handler: CPU::handleCLC, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2 },

        // CLD Instructions
        0xD8 => Operand { opcode: 0xD8, name: "CLD", handler: CPU::handleCLD, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2 },
    };

    pub(crate) fn read_u8(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub(crate) fn write_u8(& mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }

    pub(crate) fn read_u16(&self, addr: u16) -> u16 {
        // We use little-endian format: low byte at addr, high byte at addr + 1
        return u16::from_le_bytes([self.read_u8(addr), self.read_u8(addr + 1)]);
    }

    pub(crate) fn write_u16(& mut self, addr: u16, value: u16) {
        // We use little-endian format: low byte at addr, high byte at addr + 1
        let [low, high] = u16::to_le_bytes(value);

        self.write_u8(addr, low);
        self.write_u8(addr + 1, high);
    }

    pub(crate) fn set_status_flag(& mut self, flag: StatusFlag, value: bool) {
        if value {
            self.status_register |= 1 << (flag as u8);
        } else {
            self.status_register &= !(1 << (flag as u8));
        }
    }

    pub(crate) fn get_status_flag(&self, flag: StatusFlag) -> bool {
        (self.status_register & (1 << (flag as u8))) != 0
    }

    pub(crate) fn load_program(& mut self, program: &[u8]) {
        let start_address = CPU::PRG_ROM_BASE_ADDRESS as usize;
        let end_address = start_address + program.len();

        if end_address > self.memory.len() {
            panic!("Program size exceeds memory bounds");
        }

        self.memory[start_address..end_address].copy_from_slice(program);
        self.program_counter = CPU::PRG_ROM_BASE_ADDRESS;
    }

    pub(crate) fn reset(&mut self) {
        self.accumulator = 0;
        self.x_register = 0;
        self.status_register = 0;

        // 0xFFFC corresponds to the reset vector address.
        self.program_counter = self.read_u16(CPU::RESET_VECTOR_ADDRESS);
    }


    fn run(& mut self) {
        loop {
            let opcode = self.read_u8(self.program_counter);
            self.program_counter += 1;

            if let Some(operand_info) = Self::OPERAND_MAP.get(&opcode) {
                // Fetch operand based on addressing mode
                let operand_value = match operand_info.addressing_mode {
                    AddressingMode::Immediate => {
                        // For immediate addressing, read the immediate byte but don't advance PC here.
                        // We'll advance PC uniformly after executing by (bytes - 1).
                        self.read_u8(self.program_counter)
                    }
                    AddressingMode::Accumulator => self.accumulator,
                    _ => {
                        let addr = self.get_operand_address(operand_info.addressing_mode);
                        self.read_u8(addr)
                    }
                };

                // Execute the instruction and collect any additional cycles the handler returns
                let handler_extra = (operand_info.handler)(self, operand_value);

                // Add base cycles plus any additional cycles reported by handler
                self.cycles += operand_info.cycles as u64 + handler_extra as u64;

                // Update program counter (advance remaining bytes beyond opcode)
                self.program_counter += (operand_info.bytes - 1) as u16;
            } else if opcode == 0x00 {
                return;
            } else {
                panic!("Unimplemented opcode: {:02X}", opcode);
            }
        }
    }

    /// Branch helper: centralizes branch behavior for relative branches.
    /// `condition` indicates whether the branch should be taken.
    /// `offset` is the signed 8-bit relative offset.
    /// Returns additional cycles: 0 if not taken, +1 if taken, +2 if page crossed.
    pub(crate) fn branch(&mut self, condition: bool, offset: i8) -> u8 {
        let mut additional_cycles: u8 = 0;
        if condition {
            let old_pc = self.program_counter;
            self.program_counter = self.program_counter.wrapping_add(offset as u16);
            additional_cycles += 1; // branch taken
            if (old_pc & 0xFF00) != (self.program_counter & 0xFF00) {
                additional_cycles += 1; // page crossed
            }
        }
        additional_cycles
    }

    // Helper to get effective address based on addressing mode
    pub(crate) fn get_operand_address(&self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Absolute => self.read_u16(self.program_counter),

            AddressingMode::AbsoluteX => {
                let base = self.read_u16(self.program_counter);
                base.wrapping_add(self.x_register as u16)
            }

            AddressingMode::AbsoluteY => {
                let base = self.read_u16(self.program_counter);
                base.wrapping_add(self.y_register as u16)
            }

            AddressingMode::Immediate => self.program_counter,

            AddressingMode::Indirect => {
                let ptr = self.read_u16(self.program_counter);
                // 6502 hardware bug: page boundary wraps
                let low = self.read_u8(ptr);
                let high = if ptr & 0x00FF == 0x00FF {
                    // page boundary bug: wrap to beginning of same page
                    self.read_u8(ptr & 0xFF00)
                } else {
                    self.read_u8(ptr + 1)
                };
                u16::from_le_bytes([low, high])
            }

            AddressingMode::IndirectX => {
                let base = self.read_u8(self.program_counter);
                let ptr = base.wrapping_add(self.x_register);
                let low = self.read_u8(ptr as u16);
                let high = self.read_u8(ptr.wrapping_add(1) as u16);
                u16::from_le_bytes([low, high])
            }

            AddressingMode::IndirectY => {
                let base = self.read_u8(self.program_counter);
                let low = self.read_u8(base as u16);
                let high = self.read_u8(base.wrapping_add(1) as u16);
                let addr = u16::from_le_bytes([low, high]);
                addr.wrapping_add(self.y_register as u16)
            }

            AddressingMode::Relative => {
                let offset = self.read_u8(self.program_counter) as i8;
                ((self.program_counter as i16) + 1 + (offset as i16)) as u16
            }

            AddressingMode::ZeroPage => self.read_u8(self.program_counter) as u16,

            AddressingMode::ZeroPageX => {
                let base = self.read_u8(self.program_counter);
                base.wrapping_add(self.x_register) as u16
            }

            AddressingMode::ZeroPageY => {
                let base = self.read_u8(self.program_counter);
                base.wrapping_add(self.y_register) as u16
            }

            // Accumulator and Implied don't use memory addresses
            AddressingMode::Accumulator | AddressingMode::Implicit => {
                panic!("No effective address for {:?}", mode)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_init() {
        let cpu = new_cpu();
        assert_eq!(cpu.program_counter, 0x0000);
        assert_eq!(cpu.stack_pointer, 0xFF);
        assert_eq!(cpu.accumulator, 0x00);
        assert_eq!(cpu.x_register, 0x00);
        assert_eq!(cpu.y_register, 0x00);
        assert_eq!(cpu.status_register, 0x00);
        assert_eq!(cpu.memory.len(), 0xFFFF);
        for i in 0..0xFFFF {
            assert_eq!(cpu.memory[i], 0x00);
        }
    }

    // read-only helper tests: modify memory directly and verify read helpers
    #[test]
    fn test_read_u8_direct_memory() {
        let mut cpu = new_cpu();
        // Place a byte directly into memory and read it back
        cpu.memory[0x0100] = 0xAB;
        assert_eq!(cpu.read_u8(0x0100), 0xAB);

        // Overwrite directly and verify
        cpu.memory[0x0100] = 0x55;
        assert_eq!(cpu.read_u8(0x0100), 0x55);
    }

    #[test]
    fn test_read_u16_direct_memory() {
        let mut cpu = new_cpu();
        // Place low/high bytes directly and read as u16 (little-endian)
        cpu.memory[0x0200] = 0x34; // low
        cpu.memory[0x0201] = 0x12; // high
        assert_eq!(cpu.read_u16(0x0200), 0x1234);
    }

    // write-only helper tests: use write_x helpers and verify memory
    #[test]
    fn test_write_u8_writes_memory() {
        let mut cpu = new_cpu();
        cpu.write_u8(0x0100, 0xAB);
        assert_eq!(cpu.memory[0x0100], 0xAB);

        cpu.write_u8(0x0100, 0x55);
        assert_eq!(cpu.memory[0x0100], 0x55);
    }

    #[test]
    fn test_write_u16_writes_memory() {
        let mut cpu = new_cpu();
        cpu.write_u16(0x0200, 0x1234);
        // low then high (little-endian)
        assert_eq!(cpu.memory[0x0200], 0x34);
        assert_eq!(cpu.memory[0x0201], 0x12);
    }

    #[test]
    fn test_get_status_flag() {
        let mut cpu = new_cpu();

        // Test each flag by directly manipulating status_register
        for flag in [
            StatusFlag::Carry,
            StatusFlag::Zero,
            StatusFlag::InterruptDisable,
            StatusFlag::DecimalMode,
            StatusFlag::BreakCommand,
            StatusFlag::Unused,
            StatusFlag::Overflow,
            StatusFlag::Negative,
        ] {
            // Initially should be false
            assert_eq!(cpu.get_status_flag(flag), false,
                "flag {:?} should start as false", flag);

            // Set the bit directly and verify get_status_flag reads it
            cpu.status_register |= 1 << (flag as u8);
            assert_eq!(cpu.get_status_flag(flag), true,
                "flag {:?} should be true after direct set", flag);

            // Clear the bit directly and verify get_status_flag reads it
            cpu.status_register &= !(1 << (flag as u8));
            assert_eq!(cpu.get_status_flag(flag), false,
                "flag {:?} should be false after direct clear", flag);
        }
    }

    #[test]
    fn test_set_status_flag() {
        let mut cpu = new_cpu();

        // Test each flag using the set_status_flag method
        for flag in [
            StatusFlag::Carry,
            StatusFlag::Zero,
            StatusFlag::InterruptDisable,
            StatusFlag::DecimalMode,
            StatusFlag::BreakCommand,
            StatusFlag::Unused,
            StatusFlag::Overflow,
            StatusFlag::Negative,
        ] {
            // Initially should be false
            assert_eq!(cpu.status_register & (1 << (flag as u8)), 0,
                "flag {:?} bit should start as 0", flag);

            // Set to true and verify bit is set directly in status_register
            cpu.set_status_flag(flag, true);
            assert_eq!(cpu.status_register & (1 << (flag as u8)), 1 << (flag as u8),
                "flag {:?} bit should be set", flag);

            // Set to false and verify bit is cleared directly in status_register
            cpu.set_status_flag(flag, false);
            assert_eq!(cpu.status_register & (1 << (flag as u8)), 0,
                "flag {:?} bit should be cleared", flag);
        }
    }

    #[test]
    fn test_load_program() {
        let mut cpu = new_cpu();
        let program: [u8; 4] = [0x69, 0x01, 0x29, 0x02]; // ADC #$01 ; AND #$02 (example opcodes)

        // Load program and verify memory is written at PRG_ROM_BASE_ADDRESS
        cpu.load_program(&program);

        let start = CPU::PRG_ROM_BASE_ADDRESS as usize;
        for i in 0..program.len() {
            assert_eq!(cpu.memory[start + i], program[i]);
        }

        // Verify program counter was set to PRG_ROM_BASE_ADDRESS
        assert_eq!(cpu.program_counter, CPU::PRG_ROM_BASE_ADDRESS);
    }

    #[test]
    #[should_panic]
    fn test_load_program_too_big_panics() {
        let mut cpu = new_cpu();
        let start = CPU::PRG_ROM_BASE_ADDRESS as usize;
        let available = cpu.memory.len() - start;

        // Create a program that is one byte larger than the available PRG ROM space
        let program = vec![0u8; available + 1];
        cpu.load_program(&program);
    }

    #[test]
    fn test_get_operand_address() {
        let mut cpu = new_cpu();

        // Test Absolute mode
        cpu.program_counter = 0x1000;
        cpu.write_u16(0x1000, 0x3456); // Address to load
        assert_eq!(cpu.get_operand_address(AddressingMode::Absolute), 0x3456);

        // Test AbsoluteX mode
        cpu.program_counter = 0x1002;
        cpu.write_u16(0x1002, 0x3456);
        cpu.x_register = 0x10;
        assert_eq!(cpu.get_operand_address(AddressingMode::AbsoluteX), 0x3466);

        // Test AbsoluteY mode
        cpu.program_counter = 0x1004;
        cpu.write_u16(0x1004, 0x3456);
        cpu.y_register = 0x20;
        assert_eq!(cpu.get_operand_address(AddressingMode::AbsoluteY), 0x3476);

        // Test Immediate mode (returns program counter)
        cpu.program_counter = 0x1006;
        assert_eq!(cpu.get_operand_address(AddressingMode::Immediate), 0x1006);

        // Test Indirect mode
        cpu.program_counter = 0x1008;
        cpu.write_u16(0x1008, 0x2000); // Pointer address
        cpu.write_u8(0x2000, 0x34); // Low byte
        cpu.write_u8(0x2001, 0x56); // High byte
        assert_eq!(cpu.get_operand_address(AddressingMode::Indirect), 0x5634);

        // Test Indirect mode page boundary bug
        cpu.program_counter = 0x100A;
        cpu.write_u16(0x100A, 0x20FF); // Address at page boundary
        cpu.write_u8(0x20FF, 0x34); // Low byte
        cpu.write_u8(0x2000, 0x56); // High byte (wraps to start of page)
        assert_eq!(cpu.get_operand_address(AddressingMode::Indirect), 0x5634);

        // Test IndirectX mode
        cpu.program_counter = 0x100C;
        cpu.write_u8(0x100C, 0x20); // Zero page address
        cpu.x_register = 0x04;
        cpu.write_u8(0x24, 0x34); // Low byte at ($20 + X)
        cpu.write_u8(0x25, 0x56); // High byte
        assert_eq!(cpu.get_operand_address(AddressingMode::IndirectX), 0x5634);

        // Test IndirectY mode
        cpu.program_counter = 0x100E;
        cpu.write_u8(0x100E, 0x20); // Zero page address
        cpu.write_u8(0x20, 0x34); // Low byte
        cpu.write_u8(0x21, 0x56); // High byte
        cpu.y_register = 0x10;
        assert_eq!(cpu.get_operand_address(AddressingMode::IndirectY), 0x5644);

        // Test Relative mode
        cpu.program_counter = 0x1010;
        cpu.write_u8(0x1010, 0x10); // Positive offset
        assert_eq!(cpu.get_operand_address(AddressingMode::Relative), 0x1021);
        cpu.write_u8(0x1010, 0xF0); // Negative offset (-16)
        assert_eq!(cpu.get_operand_address(AddressingMode::Relative), 0x1001);

        // Test ZeroPage mode
        cpu.program_counter = 0x1011;
        cpu.write_u8(0x1011, 0x42);
        assert_eq!(cpu.get_operand_address(AddressingMode::ZeroPage), 0x0042);

        // Test ZeroPageX mode
        cpu.program_counter = 0x1012;
        cpu.write_u8(0x1012, 0x42);
        cpu.x_register = 0x08;
        assert_eq!(cpu.get_operand_address(AddressingMode::ZeroPageX), 0x004A);

        // Test ZeroPageY mode
        cpu.program_counter = 0x1013;
        cpu.write_u8(0x1013, 0x42);
        cpu.y_register = 0x09;
        assert_eq!(cpu.get_operand_address(AddressingMode::ZeroPageY), 0x004B);

        // Test Accumulator mode (should panic)
        let result = std::panic::catch_unwind(|| {
            cpu.get_operand_address(AddressingMode::Accumulator)
        });
        assert!(result.is_err());

        // Test Implicit mode (should panic)
        let result = std::panic::catch_unwind(|| {
            cpu.get_operand_address(AddressingMode::Implicit)
        });
        assert!(result.is_err());
    }

}
