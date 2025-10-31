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
    // Total memory size: 64KB; 0xFFFF + 1 = 65536 bytes = 0x10000 to include all addresses.
    memory: [u8; 0x10000],
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
        stack_pointer: CPU::STACK_ADDRESS_DEFAULT_COLD_START,
        accumulator: 0x00,
        x_register: 0x00,
        y_register: 0x00,
        status_register: 0x00,
        memory: [0; 0x10000],
        cycles: 0,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Operand {
    opcode: u8,
    name: &'static str,
    // Function pointer to the instruction handler
    //                    memory value   address
    handler: fn(&mut CPU, Option<u8>, Option<u16>) -> u8,
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
    const STACK_BASE_ADDRESS: u16 = 0x0100;
    const STACK_ADDRESS_DEFAULT_COLD_START: u8 = 0xFF;
    const STACK_ADDRESS_DEFAULT_WARM_START: u8 = 0xFD;
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

        // CLI Instructions
        0x58 => Operand { opcode: 0x58, name: "CLI", handler: CPU::handleCLI, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2 },

        // CLV Instructions
        0xB8 => Operand { opcode: 0xB8, name: "CLV", handler: CPU::handleCLV, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2 },

        // CMP Instructions
        0xC9 => Operand { opcode: 0xC9, name: "CMP", handler: CPU::handleCMP, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
        0xC5 => Operand { opcode: 0xC5, name: "CMP", handler: CPU::handleCMP, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
        0xD5 => Operand { opcode: 0xD5, name: "CMP", handler: CPU::handleCMP, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
        0xCD => Operand { opcode: 0xCD, name: "CMP", handler: CPU::handleCMP, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
        0xDD => Operand { opcode: 0xDD, name: "CMP", handler: CPU::handleCMP, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 4 /* +1 if page crossed */ },
        0xD9 => Operand { opcode: 0xD9, name: "CMP", handler: CPU::handleCMP, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 4 /* +1 if page crossed */ },
        0xC1 => Operand { opcode: 0xC1, name: "CMP", handler: CPU::handleCMP, addressing_mode: AddressingMode::IndirectX, bytes: 2, cycles: 6 },
        0xD1 => Operand { opcode: 0xD1, name: "CMP", handler: CPU::handleCMP, addressing_mode: AddressingMode::IndirectY, bytes: 2, cycles: 5 /* +1 if page crossed */ },

        // CPX Instructions
        0xE0 => Operand { opcode: 0xE0, name: "CPX", handler: CPU::handleCPX, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
        0xE4 => Operand { opcode: 0xE4, name: "CPX", handler: CPU::handleCPX, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
        0xEC => Operand { opcode: 0xEC, name: "CPX", handler: CPU::handleCPX, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },

        // CPY Instructions
        0xC0 => Operand { opcode: 0xC0, name: "CPY", handler: CPU::handleCPY, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
        0xC4 => Operand { opcode: 0xC4, name: "CPY", handler: CPU::handleCPY, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
        0xCC => Operand { opcode: 0xCC, name: "CPY", handler: CPU::handleCPY, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },

        // DEC Instructions
        0xC6 => Operand { opcode: 0xC6, name: "DEC", handler: CPU::handleDEC, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 5 },
        0xD6 => Operand { opcode: 0xD6, name: "DEC", handler: CPU::handleDEC, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 6 },
        0xCE => Operand { opcode: 0xCE, name: "DEC", handler: CPU::handleDEC, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 6 },
        0xDE => Operand { opcode: 0xDE, name: "DEC", handler: CPU::handleDEC, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 7 },

        // DEX Instructions
        0xCA => Operand { opcode: 0xCA, name: "DEX", handler: CPU::handleDEX, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2 },

        // DEY Instructions
        0x88 => Operand { opcode: 0x88, name: "DEY", handler: CPU::handleDEY, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2 },

        // EOR Instructions
        0x49 => Operand { opcode: 0x49, name: "EOR", handler: CPU::handleEOR, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
        0x45 => Operand { opcode: 0x45, name: "EOR", handler: CPU::handleEOR, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
        0x55 => Operand { opcode: 0x55, name: "EOR", handler: CPU::handleEOR, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
        0x4D => Operand { opcode: 0x4D, name: "EOR", handler: CPU::handleEOR, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
        0x5D => Operand { opcode: 0x5D, name: "EOR", handler: CPU::handleEOR, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 4 /* +1 if page crossed */ },
        0x59 => Operand { opcode: 0x59, name: "EOR", handler: CPU::handleEOR, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 4 /* +1 if page crossed */ },
        0x41 => Operand { opcode: 0x41, name: "EOR", handler: CPU::handleEOR, addressing_mode: AddressingMode::IndirectX, bytes: 2, cycles: 6 },
        0x51 => Operand { opcode: 0x51, name: "EOR", handler: CPU::handleEOR, addressing_mode: AddressingMode::IndirectY, bytes: 2, cycles: 5 /* +1 if page crossed */ },

        // INC Instructions
        0xE6 => Operand { opcode: 0xE6, name: "INC", handler: CPU::handleINC, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 5 },
        0xF6 => Operand { opcode: 0xF6, name: "INC", handler: CPU::handleINC, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 6 },
        0xEE => Operand { opcode: 0xEE, name: "INC", handler: CPU::handleINC, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 6 },
        0xFE => Operand { opcode: 0xFE, name: "INC", handler: CPU::handleINC, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 7  },

        // INX Instructions
        0xE8 => Operand { opcode: 0xE8, name: "INX", handler: CPU::handleINX, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

        // INY Instructions
        0xC8 => Operand { opcode: 0xC8, name: "INY", handler: CPU::handleINY, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

        // JMP Instructions
        0x4C => Operand { opcode: 0x4C, name: "JMP", handler: CPU::handleJMP, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 3 },
        0x6C => Operand { opcode: 0x6C, name: "JMP", handler: CPU::handleJMP, addressing_mode: AddressingMode::Indirect, bytes: 3, cycles: 5 },

        // JSR Instructions
        0x20 => Operand { opcode: 0x20, name: "JSR", handler: CPU::handleJSR, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 6  },

        // LDA Instructions
        0xA9 => Operand { opcode: 0xA9, name: "LDA", handler: CPU::handleLDA, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
        0xA5 => Operand { opcode: 0xA5, name: "LDA", handler: CPU::handleLDA, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
        0xB5 => Operand { opcode: 0xB5, name: "LDA", handler: CPU::handleLDA, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
        0xAD => Operand { opcode: 0xAD, name: "LDA", handler: CPU::handleLDA, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
        0xBD => Operand { opcode: 0xBD, name: "LDA", handler: CPU::handleLDA, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 4 /* +1 if page crossed */ },
        0xB9 => Operand { opcode: 0xB9, name: "LDA", handler: CPU::handleLDA, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 4 /* +1 if page crossed */ },
        0xA1 => Operand { opcode: 0xA1, name: "LDA", handler: CPU::handleLDA, addressing_mode: AddressingMode::IndirectX, bytes: 2, cycles: 6 },
        0xB1 => Operand { opcode: 0xB1, name: "LDA", handler: CPU::handleLDA, addressing_mode: AddressingMode::IndirectY, bytes: 2, cycles: 5 /* +1 if page crossed */  },

        // LDX Instructions
        0xA2 => Operand { opcode: 0xA2, name: "LDX", handler: CPU::handleLDX, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
        0xA6 => Operand { opcode: 0xA6, name: "LDX", handler: CPU::handleLDX, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
        0xB6 => Operand { opcode: 0xB6, name: "LDX", handler: CPU::handleLDX, addressing_mode: AddressingMode::ZeroPageY, bytes: 2, cycles: 4 },
        0xAE => Operand { opcode: 0xAE, name: "LDX", handler: CPU::handleLDX, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
        0xBE => Operand { opcode: 0xBE, name: "LDX", handler: CPU::handleLDX, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 4 /* +1 if page crossed */  },

        // LDY Instructions
        0xA0 => Operand { opcode: 0xA0, name: "LDY", handler: CPU::handleLDY, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
        0xA4 => Operand { opcode: 0xA4, name: "LDY", handler: CPU::handleLDY, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
        0xB4 => Operand { opcode: 0xB4, name: "LDY", handler: CPU::handleLDY, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
        0xAC => Operand { opcode: 0xAC, name: "LDY", handler: CPU::handleLDY, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
        0xBC => Operand { opcode: 0xBC, name: "LDY", handler: CPU::handleLDY, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 4 /* +1 if page crossed */  },

        // LSR Instructions
        0x4A => Operand { opcode: 0x4A, name: "LSR", handler: CPU::handleLSR, addressing_mode: AddressingMode::Accumulator, bytes: 1, cycles: 2 },
        0x46 => Operand { opcode: 0x46, name: "LSR", handler: CPU::handleLSR, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 5 },
        0x56 => Operand { opcode: 0x56, name: "LSR", handler: CPU::handleLSR, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 6 },
        0x4E => Operand { opcode: 0x4E, name: "LSR", handler: CPU::handleLSR, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 6 },
        0x5E => Operand { opcode: 0x5E, name: "LSR", handler: CPU::handleLSR, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 7 },

        // NOP Instructions
        0xEA => Operand { opcode: 0xEA, name: "NOP", handler: CPU::handleNOP, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2 },

        // ORA Instructions
        0x09 => Operand { opcode: 0x09, name: "ORA", handler: CPU::handleORA, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
        0x05 => Operand { opcode: 0x05, name: "ORA", handler: CPU::handleORA, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
        0x15 => Operand { opcode: 0x15, name: "ORA", handler: CPU::handleORA, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
        0x0D => Operand { opcode: 0x0D, name: "ORA", handler: CPU::handleORA, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
        0x1D => Operand { opcode: 0x1D, name: "ORA", handler: CPU::handleORA, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 4 /* +1 if page crossed */ },
        0x19 => Operand { opcode: 0x19, name: "ORA", handler: CPU::handleORA, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 4 /* +1 if page crossed */ },
        0x01 => Operand { opcode: 0x01, name: "ORA", handler: CPU::handleORA, addressing_mode: AddressingMode::IndirectX, bytes: 2, cycles: 6 },
        0x11 => Operand { opcode: 0x11, name: "ORA", handler: CPU::handleORA, addressing_mode: AddressingMode::IndirectY, bytes: 2, cycles: 5 /* +1 if page crossed */  },

        // PHA Instructions
        0x48 => Operand { opcode: 0x48, name: "PHA", handler: CPU::handlePHA, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 3  },

        // PHP Instructions
        0x08 => Operand { opcode: 0x08, name: "PHP", handler: CPU::handlePHP, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 3  },

        // PLA Instructions
        0x68 => Operand { opcode: 0x68, name: "PLA", handler: CPU::handlePLA, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 4  },

        // PLP Instructions
        0x28 => Operand { opcode: 0x28, name: "PLP", handler: CPU::handlePLP, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 4  },

        // ROL Instructions
        0x2A => Operand { opcode: 0x2A, name: "ROL", handler: CPU::handleROL, addressing_mode: AddressingMode::Accumulator, bytes: 1, cycles: 2 },
        0x26 => Operand { opcode: 0x26, name: "ROL", handler: CPU::handleROL, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 5 },
        0x36 => Operand { opcode: 0x36, name: "ROL", handler: CPU::handleROL, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 6 },
        0x2E => Operand { opcode: 0x2E, name: "ROL", handler: CPU::handleROL, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 6 },
        0x3E => Operand { opcode: 0x3E, name: "ROL", handler: CPU::handleROL, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 7 },

        // ROR Instructions
        0x6A => Operand { opcode: 0x6A, name: "ROR", handler: CPU::handleROR, addressing_mode: AddressingMode::Accumulator, bytes: 1, cycles: 2 },
        0x66 => Operand { opcode: 0x66, name: "ROR", handler: CPU::handleROR, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 5 },
        0x76 => Operand { opcode: 0x76, name: "ROR", handler: CPU::handleROR, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 6 },
        0x6E => Operand { opcode: 0x6E, name: "ROR", handler: CPU::handleROR, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 6 },
        0x7E => Operand { opcode: 0x7E, name: "ROR", handler: CPU::handleROR, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 7 },

        // RTI Instructions
        0x40 => Operand { opcode: 0x40, name: "RTI", handler: CPU::handleRTI, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 6 },

        // RTS Instructions
        0x60 => Operand { opcode: 0x60, name: "RTS", handler: CPU::handleRTS, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 6 },

        // SBC Instructions
        0xE9 => Operand { opcode: 0xE9, name: "SBC", handler: CPU::handleSBC, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
        0xE5 => Operand { opcode: 0xE5, name: "SBC", handler: CPU::handleSBC, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
        0xF5 => Operand { opcode: 0xF5, name: "SBC", handler: CPU::handleSBC, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
        0xED => Operand { opcode: 0xED, name: "SBC", handler: CPU::handleSBC, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
        0xFD => Operand { opcode: 0xFD, name: "SBC", handler: CPU::handleSBC, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 4 /* +1 if page crossed */ },
        0xF9 => Operand { opcode: 0xF9, name: "SBC", handler: CPU::handleSBC, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 4 /* +1 if page crossed */ },
        0xE1 => Operand { opcode: 0xE1, name: "SBC", handler: CPU::handleSBC, addressing_mode: AddressingMode::IndirectX, bytes: 2, cycles: 6 },
        0xF1 => Operand { opcode: 0xF1, name: "SBC", handler: CPU::handleSBC, addressing_mode: AddressingMode::IndirectY, bytes: 2, cycles: 5 /* +1 if page crossed */ },

        // SEC Instructions
        0x38 => Operand { opcode: 0x38, name: "SEC", handler: CPU::handleSEC, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

        // SED Instructions
        0xF8 => Operand { opcode: 0xF8, name: "SED", handler: CPU::handleSED, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

        // SEI Instructions
        0x78 => Operand { opcode: 0x78, name: "SEI", handler: CPU::handleSEI, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

        // STA Instructions
        0x85 => Operand { opcode: 0x85, name: "STA", handler: CPU::handleSTA, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
        0x95 => Operand { opcode: 0x95, name: "STA", handler: CPU::handleSTA, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
        0x8D => Operand { opcode: 0x8D, name: "STA", handler: CPU::handleSTA, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
        0x9D => Operand { opcode: 0x9D, name: "STA", handler: CPU::handleSTA, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 5 },
        0x99 => Operand { opcode: 0x99, name: "STA", handler: CPU::handleSTA, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 5 },
        0x81 => Operand { opcode: 0x81, name: "STA", handler: CPU::handleSTA, addressing_mode: AddressingMode::IndirectX, bytes: 2, cycles: 6 },
        0x91 => Operand { opcode: 0x91, name: "STA", handler: CPU::handleSTA, addressing_mode: AddressingMode::IndirectY, bytes: 2, cycles: 6 },

        // STX Instructions
        0x86 => Operand { opcode: 0x86, name: "STX", handler: CPU::handleSTX, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
        0x96 => Operand { opcode: 0x96, name: "STX", handler: CPU::handleSTX, addressing_mode: AddressingMode::ZeroPageY, bytes: 2, cycles: 4 },
        0x8E => Operand { opcode: 0x8E, name: "STX", handler: CPU::handleSTX, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },

        // STY Instructions
        0x84 => Operand { opcode: 0x84, name: "STY", handler: CPU::handleSTY, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
        0x94 => Operand { opcode: 0x94, name: "STY", handler: CPU::handleSTY, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
        0x8C => Operand { opcode: 0x8C, name: "STY", handler: CPU::handleSTY, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },

        // TAX Instructions
        0xAA => Operand { opcode: 0xAA, name: "TAX", handler: CPU::handleTAX, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

        // TAY Instructions
        0xA8 => Operand { opcode: 0xA8, name: "TAY", handler: CPU::handleTAY, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

        // TSX Instructions
        0xBA => Operand { opcode: 0xBA, name: "TSX", handler: CPU::handleTSX, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

        // TXA Instructions
        0x8A => Operand { opcode: 0x8A, name: "TXA", handler: CPU::handleTXA, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

        // TXS Instructions
        0x9A => Operand { opcode: 0x9A, name: "TXS", handler: CPU::handleTXS, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

        // TYA Instructions
        0x98 => Operand { opcode: 0x98, name: "TYA", handler: CPU::handleTYA, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },
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

    /// Pushes a byte onto the stack.
    pub(crate) fn push_u8(&mut self, value: u8) {
        let stack_addr = Self::STACK_BASE_ADDRESS + self.stack_pointer as u16;
        self.write_u8(stack_addr, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    /// Pushes a 16-bit word onto the stack.
    /// The high byte is pushed first, then the low byte, so they are stored in little-endian format on the stack.
    pub(crate) fn push_u16(&mut self, value: u16) {
        let [low, high] = value.to_le_bytes();
        // Push high byte first, then low byte
        self.push_u8(high);
        self.push_u8(low);
    }

    /// Pops a byte from the stack.
    pub(crate) fn pop_u8(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let stack_addr = Self::STACK_BASE_ADDRESS + self.stack_pointer as u16;
        self.read_u8(stack_addr)
    }

    /// Pops a 16-bit word from the stack.
    /// The low byte is popped first, then the high byte, as they are stored in little-endian format on the stack.
    pub(crate) fn pop_u16(&mut self) -> u16 {
        let low = self.pop_u8();
        let high = self.pop_u8();
        // Combine into a u16 value
        u16::from_le_bytes([low, high])
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
        self.stack_pointer = CPU::STACK_ADDRESS_DEFAULT_WARM_START;

        // 0xFFFC corresponds to the reset vector address.
        self.program_counter = self.read_u16(CPU::RESET_VECTOR_ADDRESS);
    }

    fn run(& mut self) {
        loop {
            let pc_before_instruction = self.program_counter;
            let opcode = self.read_u8(pc_before_instruction);

            if let Some(operand_info) = Self::OPERAND_MAP.get(&opcode) {
                // Fetch operand based on addressing mode
                let (operand_value, operand_address) = match operand_info.addressing_mode {
                    AddressingMode::Implicit => (None, None),
                    AddressingMode::Accumulator => (Some(self.accumulator), None),
                    _ => {
                        // Pass PC + 1 to get operand, as PC currently points to the opcode
                        let addr = self.get_operand_address(operand_info.addressing_mode, pc_before_instruction + 1);
                        (Some(self.read_u8(addr)), Some(addr))
                    }
                };

                // TODO: Handle the cycles better for page crossing in addressing modes that require it.
                //       Move the additional cycle of the branch instructions into this new logic.
                // Execute the instruction and collect any additional cycles the handler returns
                let handler_extra = (operand_info.handler)(self, operand_value, operand_address);

                // Add base cycles plus any additional cycles reported by handler
                self.cycles += operand_info.cycles as u64 + handler_extra as u64;

                // If the program counter was not changed by a jump or branch, advance it.
                if self.program_counter == pc_before_instruction {
                    self.program_counter += operand_info.bytes as u16;
                }
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
    pub(crate) fn get_operand_address(&self, mode: AddressingMode, addr: u16) -> u16 {
        match mode {
            AddressingMode::Absolute => self.read_u16(addr),

            AddressingMode::AbsoluteX => {
                let base = self.read_u16(addr);
                base.wrapping_add(self.x_register as u16)
            }

            AddressingMode::AbsoluteY => {
                let base = self.read_u16(addr);
                base.wrapping_add(self.y_register as u16)
            }

            AddressingMode::Immediate => addr,

            AddressingMode::Indirect => {
                let ptr = self.read_u16(addr);
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
                let base = self.read_u8(addr);
                let ptr = base.wrapping_add(self.x_register);
                let low = self.read_u8(ptr as u16);
                let high = self.read_u8(ptr.wrapping_add(1) as u16);
                u16::from_le_bytes([low, high])
            }

            AddressingMode::IndirectY => {
                let base = self.read_u8(addr);
                let low = self.read_u8(base as u16);
                let high = self.read_u8(base.wrapping_add(1) as u16);
                let addr = u16::from_le_bytes([low, high]);
                addr.wrapping_add(self.y_register as u16)
            }

            AddressingMode::Relative => {
                let offset = self.read_u8(addr) as i8;
                // The offset is relative to the address of the *next* instruction.
                addr.wrapping_add(1).wrapping_add(offset as u16)
            }

            AddressingMode::ZeroPage => self.read_u8(addr) as u16,

            AddressingMode::ZeroPageX => {
                let base = self.read_u8(addr);
                base.wrapping_add(self.x_register) as u16
            }

            AddressingMode::ZeroPageY => {
                let base = self.read_u8(addr);
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
        assert_eq!(cpu.memory.len(), 0x10000);
        for i in 0..0x10000 {
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
        let base_addr = 0x1000;

        // Test Absolute mode
        cpu.write_u16(base_addr, 0x3456); // Address to load
        assert_eq!(cpu.get_operand_address(AddressingMode::Absolute, base_addr), 0x3456);

        // Test AbsoluteX mode
        cpu.write_u16(base_addr + 2, 0x3456);
        cpu.x_register = 0x10;
        assert_eq!(cpu.get_operand_address(AddressingMode::AbsoluteX, base_addr + 2), 0x3466);

        // Test AbsoluteY mode
        cpu.write_u16(base_addr + 4, 0x3456);
        cpu.y_register = 0x20;
        assert_eq!(cpu.get_operand_address(AddressingMode::AbsoluteY, base_addr + 4), 0x3476);

        // Test Immediate mode (returns address passed in)
        assert_eq!(cpu.get_operand_address(AddressingMode::Immediate, base_addr + 6), base_addr + 6);

        // Test Indirect mode
        cpu.write_u16(base_addr + 8, 0x2000); // Pointer address
        cpu.write_u8(0x2000, 0x34); // Low byte
        cpu.write_u8(0x2001, 0x56); // High byte
        assert_eq!(cpu.get_operand_address(AddressingMode::Indirect, base_addr + 8), 0x5634);

        // Test Indirect mode page boundary bug
        cpu.write_u16(base_addr + 10, 0x20FF); // Address at page boundary
        cpu.write_u8(0x20FF, 0x34); // Low byte
        cpu.write_u8(0x2000, 0x56); // High byte (wraps to start of page)
        assert_eq!(cpu.get_operand_address(AddressingMode::Indirect, base_addr + 10), 0x5634);

        // Test IndirectX mode
        cpu.write_u8(base_addr + 12, 0x20); // Zero page address
        cpu.x_register = 0x04;
        cpu.write_u8(0x24, 0x34); // Low byte at ($20 + X)
        cpu.write_u8(0x25, 0x56); // High byte
        assert_eq!(cpu.get_operand_address(AddressingMode::IndirectX, base_addr + 12), 0x5634);

        // Test IndirectY mode
        cpu.write_u8(base_addr + 14, 0x20); // Zero page address
        cpu.write_u8(0x20, 0x34); // Low byte
        cpu.write_u8(0x21, 0x56); // High byte
        cpu.y_register = 0x10;
        assert_eq!(cpu.get_operand_address(AddressingMode::IndirectY, base_addr + 14), 0x5644);

        // Test Relative mode
        cpu.write_u8(base_addr + 16, 0x10); // Positive offset
        assert_eq!(cpu.get_operand_address(AddressingMode::Relative, base_addr + 16), base_addr + 16 + 1 + 0x10);
        cpu.write_u8(base_addr + 17, 0xF0); // Negative offset (-16)
        assert_eq!(cpu.get_operand_address(AddressingMode::Relative, base_addr + 17), (base_addr as i32 + 17 + 1 + -16) as u16);

        // Test ZeroPage mode
        cpu.write_u8(base_addr + 18, 0x42);
        assert_eq!(cpu.get_operand_address(AddressingMode::ZeroPage, base_addr + 18), 0x0042);

        // Test ZeroPageX mode
        cpu.write_u8(base_addr + 19, 0x42);
        cpu.x_register = 0x08;
        assert_eq!(cpu.get_operand_address(AddressingMode::ZeroPageX, base_addr + 19), 0x004A);

        // Test ZeroPageY mode
        cpu.write_u8(base_addr + 20, 0x42);
        cpu.y_register = 0x09;
        assert_eq!(cpu.get_operand_address(AddressingMode::ZeroPageY, base_addr + 20), 0x004B);

        // Test Accumulator mode (should panic)
        let result = std::panic::catch_unwind(|| {
            cpu.get_operand_address(AddressingMode::Accumulator, 0)
        });
        assert!(result.is_err());

        // Test Implicit mode (should panic)
        let result = std::panic::catch_unwind(|| {
            cpu.get_operand_address(AddressingMode::Implicit, 0)
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_stack_push_pop_u8() {
        let mut cpu = new_cpu();
        assert_eq!(cpu.stack_pointer, 0xFF);

        cpu.push_u8(0xAB);
        assert_eq!(cpu.stack_pointer, 0xFE);
        assert_eq!(cpu.read_u8(0x01FF), 0xAB);

        let popped_value = cpu.pop_u8();
        assert_eq!(popped_value, 0xAB);
        assert_eq!(cpu.stack_pointer, 0xFF);
    }

    #[test]
    fn test_stack_push_pop_u16() {
        let mut cpu = new_cpu();
        cpu.push_u16(0x1234);
        assert_eq!(cpu.stack_pointer, 0xFD);
        let popped_value = cpu.pop_u16();
        assert_eq!(popped_value, 0x1234);
        assert_eq!(cpu.stack_pointer, 0xFF);
    }
}
