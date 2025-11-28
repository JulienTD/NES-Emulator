use phf::phf_map;
use crate::bus::Bus;

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
    pub bus: Bus,

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

pub fn new_cpu(bus: Bus) -> CPU {
    CPU {
        program_counter: 0x0000,
        stack_pointer: CPU::STACK_ADDRESS_DEFAULT_COLD_START,
        accumulator: 0x00,
        x_register: 0x00,
        y_register: 0x00,
        status_register: 0x24, // 0010 0100 (Unused + Interrupt Disable)
        bus,
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

// List of all opcodes and their corresponding Operand definitions.
static OPERAND_MAP: phf::Map<u8, Operand> = phf_map! {
    // ADC Instructions
    0x69u8 => Operand { opcode: 0x69, name: "ADC", handler: CPU::handle_adc, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
    0x65u8 => Operand { opcode: 0x65, name: "ADC", handler: CPU::handle_adc, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
    0x75u8 => Operand { opcode: 0x75, name: "ADC", handler: CPU::handle_adc, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
    0x6Du8 => Operand { opcode: 0x6D, name: "ADC", handler: CPU::handle_adc, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
    0x7Du8 => Operand { opcode: 0x7D, name: "ADC", handler: CPU::handle_adc, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 4 /* +1 if page crossed */ },
    0x79u8 => Operand { opcode: 0x79, name: "ADC", handler: CPU::handle_adc, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 4 /* +1 if page crossed */ },
    0x61u8 => Operand { opcode: 0x61, name: "ADC", handler: CPU::handle_adc, addressing_mode: AddressingMode::IndirectX, bytes: 2, cycles: 6 },
    0x71u8 => Operand { opcode: 0x71, name: "ADC", handler: CPU::handle_adc, addressing_mode: AddressingMode::IndirectY, bytes: 2, cycles: 5 /* +1 if page crossed */ },

    // AND Instructions
    0x29u8 => Operand { opcode: 0x29, name: "AND", handler: CPU::handle_and, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
    0x25u8 => Operand { opcode: 0x25, name: "AND", handler: CPU::handle_and, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
    0x35u8 => Operand { opcode: 0x35, name: "AND", handler: CPU::handle_and, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
    0x2Du8 => Operand { opcode: 0x2D, name: "AND", handler: CPU::handle_and, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
    0x3Du8 => Operand { opcode: 0x3D, name: "AND", handler: CPU::handle_and, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 4 /* +1 if page crossed */ }, // TODO
    0x39u8 => Operand { opcode: 0x39, name: "AND", handler: CPU::handle_and, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 4 /* +1 if page crossed */ },
    0x21u8 => Operand { opcode: 0x21, name: "AND", handler: CPU::handle_and, addressing_mode: AddressingMode::IndirectX, bytes: 2, cycles: 6 },
    0x31u8 => Operand { opcode: 0x31, name: "AND", handler: CPU::handle_and, addressing_mode: AddressingMode::IndirectY, bytes: 2, cycles: 5 /* +1 if page crossed */ },

    // ASL Instructions
    0x0Au8 => Operand { opcode: 0x0A, name: "ASL", handler: CPU::handle_asl, addressing_mode: AddressingMode::Accumulator, bytes: 1, cycles: 2 },
    0x06u8 => Operand { opcode: 0x06, name: "ASL", handler: CPU::handle_asl, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 5 },
    0x16u8 => Operand { opcode: 0x16, name: "ASL", handler: CPU::handle_asl, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 6 },
    0x0Eu8 => Operand { opcode: 0x0E, name: "ASL", handler: CPU::handle_asl, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 6 },
    0x1Eu8 => Operand { opcode: 0x1E, name: "ASL", handler: CPU::handle_asl, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 7 },

    // BCC Instructions
    0x90u8 => Operand { opcode: 0x90, name: "BCC", handler: CPU::handle_bcc, addressing_mode: AddressingMode::Relative, bytes: 2, cycles: 2 /* +1 if branch succeeds or +2 if to a new page */ },

    // BCS Instructions
    0xB0u8 => Operand { opcode: 0xB0, name: "BCS", handler: CPU::handle_bcs, addressing_mode: AddressingMode::Relative, bytes: 2, cycles: 2 /* +1 if branch succeeds or +2 if to a new page */ },

    // BEQ Instructions
    0xF0u8 => Operand { opcode: 0xF0, name: "BEQ", handler: CPU::handle_beq, addressing_mode: AddressingMode::Relative, bytes: 2, cycles: 2 /* +1 if branch succeeds or +2 if to a new page */ },

    // BIT Instructions
    0x24u8 => Operand { opcode: 0x24, name: "BIT", handler: CPU::handle_bit, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
    0x2Cu8 => Operand { opcode: 0x2C, name: "BIT", handler: CPU::handle_bit, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },

    // BMI Instructions
    0x30u8 => Operand { opcode: 0x30, name: "BMI", handler: CPU::handle_bmi, addressing_mode: AddressingMode::Relative, bytes: 2, cycles: 2 /* +1 if branch succeeds or +2 if to a new page */  },

    // BNE Instructions
    0xD0u8 => Operand { opcode: 0xD0, name: "BNE", handler: CPU::handle_bne, addressing_mode: AddressingMode::Relative, bytes: 2, cycles: 2 /* +1 if branch succeeds or +2 if to a new page */  },

    // BPL Instructions
    0x10u8 => Operand { opcode: 0x10, name: "BPL", handler: CPU::handle_bpl, addressing_mode: AddressingMode::Relative, bytes: 2, cycles: 2 /* +1 if branch succeeds or +2 if to a new page */  },

    // BRK Instructions
    0x00u8 => Operand { opcode: 0x00, name: "BRK", handler: CPU::handle_brk, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 7 },

    // BVC Instructions
    0x50u8 => Operand { opcode: 0x50, name: "BVC", handler: CPU::handle_bvc, addressing_mode: AddressingMode::Relative, bytes: 2, cycles: 2 /* +1 if branch succeeds or +2 if to a new page */  },

    // BVS Instructions
    0x70u8 => Operand { opcode: 0x70, name: "BVS", handler: CPU::handle_bvs, addressing_mode: AddressingMode::Relative, bytes: 2, cycles: 2 /* +1 if branch succeeds or +2 if to a new page */  },

    // CLC Instructions
    0x18u8 => Operand { opcode: 0x18, name: "CLC", handler: CPU::handle_clc, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2 },

    // CLD Instructions
    0xD8u8 => Operand { opcode: 0xD8, name: "CLD", handler: CPU::handle_cld, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2 },

    // CLI Instructions
    0x58u8 => Operand { opcode: 0x58, name: "CLI", handler: CPU::handle_cli, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2 },

    // CLV Instructions
    0xB8u8 => Operand { opcode: 0xB8, name: "CLV", handler: CPU::handle_clv, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2 },

    // CMP Instructions
    0xC9u8 => Operand { opcode: 0xC9, name: "CMP", handler: CPU::handle_cmp, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
    0xC5u8 => Operand { opcode: 0xC5, name: "CMP", handler: CPU::handle_cmp, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
    0xD5u8 => Operand { opcode: 0xD5, name: "CMP", handler: CPU::handle_cmp, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
    0xCDu8 => Operand { opcode: 0xCD, name: "CMP", handler: CPU::handle_cmp, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
    0xDDu8 => Operand { opcode: 0xDD, name: "CMP", handler: CPU::handle_cmp, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 4 /* +1 if page crossed */ },
    0xD9u8 => Operand { opcode: 0xD9, name: "CMP", handler: CPU::handle_cmp, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 4 /* +1 if page crossed */ },
    0xC1u8 => Operand { opcode: 0xC1, name: "CMP", handler: CPU::handle_cmp, addressing_mode: AddressingMode::IndirectX, bytes: 2, cycles: 6 },
    0xD1u8 => Operand { opcode: 0xD1, name: "CMP", handler: CPU::handle_cmp, addressing_mode: AddressingMode::IndirectY, bytes: 2, cycles: 5 /* +1 if page crossed */ },

    // CPX Instructions
    0xE0u8 => Operand { opcode: 0xE0, name: "CPX", handler: CPU::handle_cpx, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
    0xE4u8 => Operand { opcode: 0xE4, name: "CPX", handler: CPU::handle_cpx, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
    0xECu8 => Operand { opcode: 0xEC, name: "CPX", handler: CPU::handle_cpx, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },

    // CPY Instructions
    0xC0u8 => Operand { opcode: 0xC0, name: "CPY", handler: CPU::handle_cpy, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
    0xC4u8 => Operand { opcode: 0xC4, name: "CPY", handler: CPU::handle_cpy, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
    0xCCu8 => Operand { opcode: 0xCC, name: "CPY", handler: CPU::handle_cpy, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },

    // DEC Instructions
    0xC6u8 => Operand { opcode: 0xC6, name: "DEC", handler: CPU::handle_dec, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 5 },
    0xD6u8 => Operand { opcode: 0xD6, name: "DEC", handler: CPU::handle_dec, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 6 },
    0xCEu8 => Operand { opcode: 0xCE, name: "DEC", handler: CPU::handle_dec, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 6 },
    0xDEu8 => Operand { opcode: 0xDE, name: "DEC", handler: CPU::handle_dec, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 7 },

    // DEX Instructions
    0xCAu8 => Operand { opcode: 0xCA, name: "DEX", handler: CPU::handle_dex, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2 },

    // DEY Instructions
    0x88u8 => Operand { opcode: 0x88, name: "DEY", handler: CPU::handle_dey, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2 },

    // EOR Instructions
    0x49u8 => Operand { opcode: 0x49, name: "EOR", handler: CPU::handle_eor, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
    0x45u8 => Operand { opcode: 0x45, name: "EOR", handler: CPU::handle_eor, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
    0x55u8 => Operand { opcode: 0x55, name: "EOR", handler: CPU::handle_eor, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
    0x4Du8 => Operand { opcode: 0x4D, name: "EOR", handler: CPU::handle_eor, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
    0x5Du8 => Operand { opcode: 0x5D, name: "EOR", handler: CPU::handle_eor, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 4 /* +1 if page crossed */ },
    0x59u8 => Operand { opcode: 0x59, name: "EOR", handler: CPU::handle_eor, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 4 /* +1 if page crossed */ },
    0x41u8 => Operand { opcode: 0x41, name: "EOR", handler: CPU::handle_eor, addressing_mode: AddressingMode::IndirectX, bytes: 2, cycles: 6 },
    0x51u8 => Operand { opcode: 0x51, name: "EOR", handler: CPU::handle_eor, addressing_mode: AddressingMode::IndirectY, bytes: 2, cycles: 5 /* +1 if page crossed */ },

    // INC Instructions
    0xE6u8 => Operand { opcode: 0xE6, name: "INC", handler: CPU::handle_inc, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 5 },
    0xF6u8 => Operand { opcode: 0xF6, name: "INC", handler: CPU::handle_inc, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 6 },
    0xEEu8 => Operand { opcode: 0xEE, name: "INC", handler: CPU::handle_inc, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 6 },
    0xFEu8 => Operand { opcode: 0xFE, name: "INC", handler: CPU::handle_inc, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 7  },

    // INX Instructions
    0xE8u8 => Operand { opcode: 0xE8, name: "INX", handler: CPU::handle_inx, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

    // INY Instructions
    0xC8u8 => Operand { opcode: 0xC8, name: "INY", handler: CPU::handle_iny, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

    // JMP Instructions
    0x4Cu8 => Operand { opcode: 0x4C, name: "JMP", handler: CPU::handle_jmp, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 3 },
    0x6Cu8 => Operand { opcode: 0x6C, name: "JMP", handler: CPU::handle_jmp, addressing_mode: AddressingMode::Indirect, bytes: 3, cycles: 5 },

    // JSR Instructions
    0x20u8 => Operand { opcode: 0x20, name: "JSR", handler: CPU::handle_jsr, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 6  },

    // LDA Instructions
    0xA9u8 => Operand { opcode: 0xA9, name: "LDA", handler: CPU::handle_lda, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
    0xA5u8 => Operand { opcode: 0xA5, name: "LDA", handler: CPU::handle_lda, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
    0xB5u8 => Operand { opcode: 0xB5, name: "LDA", handler: CPU::handle_lda, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
    0xADu8 => Operand { opcode: 0xAD, name: "LDA", handler: CPU::handle_lda, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
    0xBDu8 => Operand { opcode: 0xBD, name: "LDA", handler: CPU::handle_lda, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 4 /* +1 if page crossed */ },
    0xB9u8 => Operand { opcode: 0xB9, name: "LDA", handler: CPU::handle_lda, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 4 /* +1 if page crossed */ },
    0xA1u8 => Operand { opcode: 0xA1, name: "LDA", handler: CPU::handle_lda, addressing_mode: AddressingMode::IndirectX, bytes: 2, cycles: 6 },
    0xB1u8 => Operand { opcode: 0xB1, name: "LDA", handler: CPU::handle_lda, addressing_mode: AddressingMode::IndirectY, bytes: 2, cycles: 5 /* +1 if page crossed */  },

    // LDX Instructions
    0xA2u8 => Operand { opcode: 0xA2, name: "LDX", handler: CPU::handle_ldx, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
    0xA6u8 => Operand { opcode: 0xA6, name: "LDX", handler: CPU::handle_ldx, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
    0xB6u8 => Operand { opcode: 0xB6, name: "LDX", handler: CPU::handle_ldx, addressing_mode: AddressingMode::ZeroPageY, bytes: 2, cycles: 4 },
    0xAEu8 => Operand { opcode: 0xAE, name: "LDX", handler: CPU::handle_ldx, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
    0xBEu8 => Operand { opcode: 0xBE, name: "LDX", handler: CPU::handle_ldx, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 4 /* +1 if page crossed */  },

    // LDY Instructions
    0xA0u8 => Operand { opcode: 0xA0, name: "LDY", handler: CPU::handle_ldy, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
    0xA4u8 => Operand { opcode: 0xA4, name: "LDY", handler: CPU::handle_ldy, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
    0xB4u8 => Operand { opcode: 0xB4, name: "LDY", handler: CPU::handle_ldy, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
    0xACu8 => Operand { opcode: 0xAC, name: "LDY", handler: CPU::handle_ldy, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
    0xBCu8 => Operand { opcode: 0xBC, name: "LDY", handler: CPU::handle_ldy, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 4 /* +1 if page crossed */  },

    // LSR Instructions
    0x4Au8 => Operand { opcode: 0x4A, name: "LSR", handler: CPU::handle_lsr, addressing_mode: AddressingMode::Accumulator, bytes: 1, cycles: 2 },
    0x46u8 => Operand { opcode: 0x46, name: "LSR", handler: CPU::handle_lsr, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 5 },
    0x56u8 => Operand { opcode: 0x56, name: "LSR", handler: CPU::handle_lsr, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 6 },
    0x4Eu8 => Operand { opcode: 0x4E, name: "LSR", handler: CPU::handle_lsr, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 6 },
    0x5Eu8 => Operand { opcode: 0x5E, name: "LSR", handler: CPU::handle_lsr, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 7 },

    // NOP Instructions
    0xEAu8 => Operand { opcode: 0xEA, name: "NOP", handler: CPU::handle_nop, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2 },

    // ORA Instructions
    0x09u8 => Operand { opcode: 0x09, name: "ORA", handler: CPU::handle_ora, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
    0x05u8 => Operand { opcode: 0x05, name: "ORA", handler: CPU::handle_ora, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
    0x15u8 => Operand { opcode: 0x15, name: "ORA", handler: CPU::handle_ora, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
    0x0Du8 => Operand { opcode: 0x0D, name: "ORA", handler: CPU::handle_ora, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
    0x1Du8 => Operand { opcode: 0x1D, name: "ORA", handler: CPU::handle_ora, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 4 /* +1 if page crossed */ },
    0x19u8 => Operand { opcode: 0x19, name: "ORA", handler: CPU::handle_ora, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 4 /* +1 if page crossed */ },
    0x01u8 => Operand { opcode: 0x01, name: "ORA", handler: CPU::handle_ora, addressing_mode: AddressingMode::IndirectX, bytes: 2, cycles: 6 },
    0x11u8 => Operand { opcode: 0x11, name: "ORA", handler: CPU::handle_ora, addressing_mode: AddressingMode::IndirectY, bytes: 2, cycles: 5 /* +1 if page crossed */  },

    // PHA Instructions
    0x48u8 => Operand { opcode: 0x48, name: "PHA", handler: CPU::handle_pha, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 3  },

    // PHP Instructions
    0x08u8 => Operand { opcode: 0x08, name: "PHP", handler: CPU::handle_php, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 3  },

    // PLA Instructions
    0x68u8 => Operand { opcode: 0x68, name: "PLA", handler: CPU::handle_pla, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 4  },

    // PLP Instructions
    0x28u8 => Operand { opcode: 0x28, name: "PLP", handler: CPU::handle_plp, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 4  },

    // ROL Instructions
    0x2Au8 => Operand { opcode: 0x2A, name: "ROL", handler: CPU::handle_rol, addressing_mode: AddressingMode::Accumulator, bytes: 1, cycles: 2 },
    0x26u8 => Operand { opcode: 0x26, name: "ROL", handler: CPU::handle_rol, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 5 },
    0x36u8 => Operand { opcode: 0x36, name: "ROL", handler: CPU::handle_rol, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 6 },
    0x2Eu8 => Operand { opcode: 0x2E, name: "ROL", handler: CPU::handle_rol, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 6 },
    0x3Eu8 => Operand { opcode: 0x3E, name: "ROL", handler: CPU::handle_rol, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 7 },

    // ROR Instructions
    0x6Au8 => Operand { opcode: 0x6A, name: "ROR", handler: CPU::handle_ror, addressing_mode: AddressingMode::Accumulator, bytes: 1, cycles: 2 },
    0x66u8 => Operand { opcode: 0x66, name: "ROR", handler: CPU::handle_ror, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 5 },
    0x76u8 => Operand { opcode: 0x76, name: "ROR", handler: CPU::handle_ror, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 6 },
    0x6Eu8 => Operand { opcode: 0x6E, name: "ROR", handler: CPU::handle_ror, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 6 },
    0x7Eu8 => Operand { opcode: 0x7E, name: "ROR", handler: CPU::handle_ror, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 7 },

    // RTI Instructions
    0x40u8 => Operand { opcode: 0x40, name: "RTI", handler: CPU::handle_rti, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 6 },

    // RTS Instructions
    0x60u8 => Operand { opcode: 0x60, name: "RTS", handler: CPU::handle_rts, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 6 },

    // SBC Instructions
    0xE9u8 => Operand { opcode: 0xE9, name: "SBC", handler: CPU::handle_sbc, addressing_mode: AddressingMode::Immediate, bytes: 2, cycles: 2 },
    0xE5u8 => Operand { opcode: 0xE5, name: "SBC", handler: CPU::handle_sbc, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
    0xF5u8 => Operand { opcode: 0xF5, name: "SBC", handler: CPU::handle_sbc, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
    0xEDu8 => Operand { opcode: 0xED, name: "SBC", handler: CPU::handle_sbc, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
    0xFDu8 => Operand { opcode: 0xFD, name: "SBC", handler: CPU::handle_sbc, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 4 /* +1 if page crossed */ },
    0xF9u8 => Operand { opcode: 0xF9, name: "SBC", handler: CPU::handle_sbc, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 4 /* +1 if page crossed */ },
    0xE1u8 => Operand { opcode: 0xE1, name: "SBC", handler: CPU::handle_sbc, addressing_mode: AddressingMode::IndirectX, bytes: 2, cycles: 6 },
    0xF1u8 => Operand { opcode: 0xF1, name: "SBC", handler: CPU::handle_sbc, addressing_mode: AddressingMode::IndirectY, bytes: 2, cycles: 5 /* +1 if page crossed */ },

    // SEC Instructions
    0x38u8 => Operand { opcode: 0x38, name: "SEC", handler: CPU::handle_sec, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

    // SED Instructions
    0xF8u8 => Operand { opcode: 0xF8, name: "SED", handler: CPU::handle_sed, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

    // SEI Instructions
    0x78u8 => Operand { opcode: 0x78, name: "SEI", handler: CPU::handle_sei, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

    // STA Instructions
    0x85u8 => Operand { opcode: 0x85, name: "STA", handler: CPU::handle_sta, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
    0x95u8 => Operand { opcode: 0x95, name: "STA", handler: CPU::handle_sta, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
    0x8Du8 => Operand { opcode: 0x8D, name: "STA", handler: CPU::handle_sta, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },
    0x9Du8 => Operand { opcode: 0x9D, name: "STA", handler: CPU::handle_sta, addressing_mode: AddressingMode::AbsoluteX, bytes: 3, cycles: 5 },
    0x99u8 => Operand { opcode: 0x99, name: "STA", handler: CPU::handle_sta, addressing_mode: AddressingMode::AbsoluteY, bytes: 3, cycles: 5 },
    0x81u8 => Operand { opcode: 0x81, name: "STA", handler: CPU::handle_sta, addressing_mode: AddressingMode::IndirectX, bytes: 2, cycles: 6 },
    0x91u8 => Operand { opcode: 0x91, name: "STA", handler: CPU::handle_sta, addressing_mode: AddressingMode::IndirectY, bytes: 2, cycles: 6 },

    // STX Instructions
    0x86u8 => Operand { opcode: 0x86, name: "STX", handler: CPU::handle_stx, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
    0x96u8 => Operand { opcode: 0x96, name: "STX", handler: CPU::handle_stx, addressing_mode: AddressingMode::ZeroPageY, bytes: 2, cycles: 4 },
    0x8Eu8 => Operand { opcode: 0x8E, name: "STX", handler: CPU::handle_stx, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },

    // STY Instructions
    0x84u8 => Operand { opcode: 0x84, name: "STY", handler: CPU::handle_sty, addressing_mode: AddressingMode::ZeroPage, bytes: 2, cycles: 3 },
    0x94u8 => Operand { opcode: 0x94, name: "STY", handler: CPU::handle_sty, addressing_mode: AddressingMode::ZeroPageX, bytes: 2, cycles: 4 },
    0x8Cu8 => Operand { opcode: 0x8C, name: "STY", handler: CPU::handle_sty, addressing_mode: AddressingMode::Absolute, bytes: 3, cycles: 4 },

    // TAX Instructions
    0xAAu8 => Operand { opcode: 0xAA, name: "TAX", handler: CPU::handle_tax, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

    // TAY Instructions
    0xA8u8 => Operand { opcode: 0xA8, name: "TAY", handler: CPU::handle_tay, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

    // TSX Instructions
    0xBAu8 => Operand { opcode: 0xBA, name: "TSX", handler: CPU::handle_tsx, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

    // TXA Instructions
    0x8Au8 => Operand { opcode: 0x8A, name: "TXA", handler: CPU::handle_txa, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

    // TXS Instructions
    0x9Au8 => Operand { opcode: 0x9A, name: "TXS", handler: CPU::handle_txs, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },

    // TYA Instructions
    0x98u8 => Operand { opcode: 0x98, name: "TYA", handler: CPU::handle_tya, addressing_mode: AddressingMode::Implicit, bytes: 1, cycles: 2  },
};

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

    pub(crate) fn read_u8(&self, addr: u16) -> u8 {
        self.bus.read_u8(addr)
    }

    pub(crate) fn write_u8(& mut self, addr: u16, value: u8) {
        self.bus.write_u8(addr, value);
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
        // let start_address = CPU::PRG_ROM_BASE_ADDRESS as usize;
        // let end_address = start_address + program.len();

        // if end_address > self.memory.len() {
        //     panic!("Program size exceeds memory bounds");
        // }

        for i in 0..(program.len() as u16) {
            self.write_u8(0x0000 + i, program[i as usize]);
        }
        self.write_u16(0xFFFC, 0x0000); // Set reset vector to start of program
        self.program_counter = self.read_u16(CPU::RESET_VECTOR_ADDRESS);
    }

    pub(crate) fn reset(&mut self) {
        self.accumulator = 0;
        self.x_register = 0;
        self.status_register = 0x24; // 0010 0100 (Unused + Interrupt Disable)
        self.stack_pointer = CPU::STACK_ADDRESS_DEFAULT_WARM_START;

        // 0xFFFC corresponds to the reset vector address.
        self.program_counter = self.read_u16(CPU::RESET_VECTOR_ADDRESS);
        self.cycles = 8; // Reset takes 8 cycles
    }

    // Helper function to check if two addresses are on different pages
    pub(crate) fn page_crossed(&self, addr1: u16, addr2: u16) -> bool {
        (addr1 & 0xFF00) != (addr2 & 0xFF00)
    }

    pub fn run(& mut self) {
                self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        loop {
            callback(self);
            let pc_before_instruction = self.program_counter;
            let opcode = self.read_u8(pc_before_instruction);
            // println!("PC: {:04X} Opcode: {:02X}", pc_before_instruction, opcode);

            if let Some(operand_info) = OPERAND_MAP.get(&opcode) {
                // Fetch operand based on addressing mode
                let (operand_value, operand_address) = match operand_info.addressing_mode {
                    AddressingMode::Implicit => (None, None),
                    AddressingMode::Accumulator => (Some(self.accumulator), None),
                    _ => {
                        // Pass PC + 1 to get operand, as PC currently points to the opcode
                        let (addr, page_crossed) = self.get_operand_address(operand_info.addressing_mode, pc_before_instruction + 1);
                        if page_crossed {
                            match operand_info.name {
                                "ADC" | "AND" | "CMP" | "EOR" | "LDA" | "LDX" | "LDY" | "ORA" | "SBC" => {
                                    self.cycles += 1;
                                }
                                // "STA", "STX", "STY" and others do not take the penalty
                                _ => {}
                            }
                        }
                        (Some(self.read_u8(addr)), Some(addr))
                    }
                };

                // Execute the instruction and collect any additional cycles the handler returns
                let handler_extra = (operand_info.handler)(self, operand_value, operand_address);

                // Add base cycles plus any additional cycles reported by handler
                self.cycles += operand_info.cycles as u64 + handler_extra as u64;

                // If the program counter was not changed by a jump or branch, advance it.
                if self.program_counter == pc_before_instruction {
                    self.program_counter += operand_info.bytes as u16;
                }
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
            // Branch is always relative to the next instruction (PC + 2)
            let pc_next = self.program_counter.wrapping_add(2);
            let target_pc = pc_next.wrapping_add(offset as u16);

            self.program_counter = target_pc;

            additional_cycles += 1; // branch taken

            // Page crossing is compared between Next PC and Target PC
            if self.page_crossed(pc_next, target_pc) {
                additional_cycles += 1; // page crossed
            }
        }
        additional_cycles
    }

    // Helper to get effective address based on addressing mode
    pub(crate) fn get_operand_address(&self, mode: AddressingMode, addr: u16) -> (u16, bool) {
        match mode {
            AddressingMode::Absolute => (self.read_u16(addr), false),

            AddressingMode::AbsoluteX => {
                let base = self.read_u16(addr);
                let final_addr = base.wrapping_add(self.x_register as u16);
                (final_addr, self.page_crossed(base, final_addr))
            }

            AddressingMode::AbsoluteY => {
                let base = self.read_u16(addr);
                let final_addr = base.wrapping_add(self.y_register as u16);
                (final_addr, self.page_crossed(base, final_addr))
            }

            AddressingMode::Immediate => (addr, false),

            AddressingMode::Indirect => {
                let ptr = self.read_u16(addr);
                let low = self.read_u8(ptr);
                let high = if ptr & 0x00FF == 0x00FF {
                    // page boundary bug: wrap to beginning of same page
                    self.read_u8(ptr & 0xFF00)
                } else {
                    self.read_u8(ptr + 1)
                };
                (u16::from_le_bytes([low, high]), false)
            }

            AddressingMode::IndirectX => {
                let base = self.read_u8(addr);
                let ptr = base.wrapping_add(self.x_register);
                let low = self.read_u8(ptr as u16);
                let high = self.read_u8(ptr.wrapping_add(1) as u16);
                (u16::from_le_bytes([low, high]), false)
            }

            AddressingMode::IndirectY => {
                let base = self.read_u8(addr);
                let low = self.read_u8(base as u16);
                let high = self.read_u8(base.wrapping_add(1) as u16);
                let base_addr = u16::from_le_bytes([low, high]);
                let final_addr = base_addr.wrapping_add(self.y_register as u16);
                (final_addr, self.page_crossed(base_addr, final_addr))
            }

            AddressingMode::Relative => {
                (addr, false)
            }

            AddressingMode::ZeroPage => (self.read_u8(addr) as u16, false),

            AddressingMode::ZeroPageX => {
                let base = self.read_u8(addr);
                (base.wrapping_add(self.x_register) as u16, false)
            }

            AddressingMode::ZeroPageY => {
                let base = self.read_u8(addr);
                (base.wrapping_add(self.y_register) as u16, false)
            }

            // Accumulator and Implicit don't use memory addresses
            AddressingMode::Accumulator | AddressingMode::Implicit => {
                panic!("No effective address for {:?}", mode)
            }
        }
    }
}

pub fn trace(cpu: &mut CPU) -> String {
    let pc = cpu.program_counter;
    let code = cpu.read_u8(pc);
    let ops = OPERAND_MAP.get(&code).expect(&format!("Opcode {:x} is not supported", code));

    let mut hex_dump = vec![];
    hex_dump.push(code);

    let (mem_addr, stored_value) = match ops.addressing_mode {
        AddressingMode::Immediate | AddressingMode::Implicit | AddressingMode::Accumulator => (0, 0),
        _ => {
            let (addr, _) = cpu.get_operand_address(ops.addressing_mode, pc + 1);
            (addr, cpu.read_u8(addr))
        }
    };

    let tmp_ops = match ops.bytes {
        1 => match ops.opcode {
            0x0A | 0x4A | 0x2A | 0x6A => "A ".to_string(), // Accumulator instructions
            _ => String::from("")
        },
        2 => {
            let address: u8 = cpu.read_u8(pc + 1);
            hex_dump.push(address);

            match ops.addressing_mode {
                AddressingMode::Immediate => format!("#${:02X}", address),
                AddressingMode::ZeroPage => format!("${:02X} = {:02X}", mem_addr, stored_value),
                AddressingMode::ZeroPageX => format!("${:02X},X @ {:02X} = {:02X}", address, mem_addr, stored_value),
                AddressingMode::ZeroPageY => format!("${:02X},Y @ {:02X} = {:02X}", address, mem_addr, stored_value),
                AddressingMode::IndirectX => format!("(${:02X},X) @ {:02X} = {:04X} = {:02X}", address, (address.wrapping_add(cpu.x_register)), mem_addr, stored_value),
                AddressingMode::IndirectY => format!("(${:02X}),Y = {:04X} @ {:04X} = {:02X}", address, (mem_addr.wrapping_sub(cpu.y_register as u16)), mem_addr, stored_value),
                AddressingMode::Relative => {
                    let offset = cpu.read_u8(pc + 1) as i8;
                    let target = pc.wrapping_add(2).wrapping_add(offset as u16);
                    format!("${:04X}", target)
                },
                _ => panic!("Unexpected addressing mode {:?} for 2 byte instruction", ops.addressing_mode),
            }
        },
        3 => {
            let address_lo = cpu.read_u8(pc + 1);
            let address_hi = cpu.read_u8(pc + 2);
            hex_dump.push(address_lo);
            hex_dump.push(address_hi);

            let address = cpu.read_u16(pc + 1);

            match ops.addressing_mode {
                AddressingMode::Absolute => {
                     if ops.opcode == 0x4C || ops.opcode == 0x20 { // JMP, JSR
                         format!("${:04X}", mem_addr)
                     } else {
                         format!("${:04X} = {:02X}", mem_addr, stored_value)
                     }
                },
                AddressingMode::AbsoluteX => format!("${:04X},X @ {:04X} = {:02X}", address, mem_addr, stored_value),
                AddressingMode::AbsoluteY => format!("${:04X},Y @ {:04X} = {:02X}", address, mem_addr, stored_value),
                AddressingMode::Indirect => { // JMP Indirect
                    let jump_addr = if address & 0x00FF == 0x00FF {
                        let lo = cpu.read_u8(address);
                        let hi = cpu.read_u8(address & 0xFF00);
                        u16::from_le_bytes([lo, hi])
                    } else {
                        cpu.read_u16(address)
                    };
                    format!("(${:04X}) = {:04X}", address, jump_addr)
                },
                _ => panic!("Unexpected addressing mode {:?} for 3 byte instruction", ops.addressing_mode),
            }
        },
        _ => String::from("")
    };

    let hex_str = hex_dump.iter()
        .map(|z| format!("{:02X}", z))
        .collect::<Vec<String>>()
        .join(" ");

    // Nestest alignment:
    // 00-03: PC
    // 04-05: Space
    // 06-14: Hex (9 chars left aligned)
    // 15:    Space
    // 16-47: Assembly
    // 48...: Registers
    let asm_str = format!("{:04X}  {:<8} {: >4} {}", pc, hex_str, ops.name, tmp_ops)
        .trim()
        .to_string();

    format!(
        "{:47} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:  0,  0 CYC:{}", 
        asm_str, cpu.accumulator, cpu.x_register, cpu.y_register, cpu.status_register, cpu.stack_pointer, cpu.cycles
    ).to_uppercase()
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu6502::{AddressingMode, new_cpu, StatusFlag};
    use crate::rom::Rom;

    #[test]
    fn test_cpu_init() {
        let cpu = new_cpu(Bus::new(Rom::test_rom()));
        assert_eq!(cpu.program_counter, 0x0000);
        assert_eq!(cpu.stack_pointer, 0xFF);
        assert_eq!(cpu.accumulator, 0x00);
        assert_eq!(cpu.x_register, 0x00);
        assert_eq!(cpu.y_register, 0x00);
        assert_eq!(cpu.status_register, 0x24);
    }

    #[test]
    fn test_get_status_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));

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
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));

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

    // #[test]
    // fn test_load_program() {
    //     let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
    //     let program: [u8; 4] = [0x69, 0x01, 0x29, 0x02]; // ADC #$01 ; AND #$02 (example opcodes)

    //     // Load program and verify memory is written at PRG_ROM_BASE_ADDRESS
    //     cpu.load_program(&program);

    //     let start = CPU::PRG_ROM_BASE_ADDRESS as usize;
    //     for i in 0..program.len() {
    //         assert_eq!(cpu.memory[start + i], program[i]);
    //     }

    //     // Verify program counter was set to PRG_ROM_BASE_ADDRESS
    //     assert_eq!(cpu.program_counter, CPU::PRG_ROM_BASE_ADDRESS);
    // }

    // #[test]
    // #[should_panic]
    // fn test_load_program_too_big_panics() {
    //     let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
    //     let start = CPU::PRG_ROM_BASE_ADDRESS as usize;
    //     let available = 2048 - start;

    //     // Create a program that is one byte larger than the available PRG ROM space
    //     let program = vec![0u8; available + 1];
    //     cpu.load_program(&program);
    // }

    #[test]
    fn test_get_operand_address() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let instruction_ptr = 0x1000;

        // 1. Absolute: (Never crosses)
        cpu.write_u16(instruction_ptr, 0x3456);
        assert_eq!(
            cpu.get_operand_address(AddressingMode::Absolute, instruction_ptr),
            (0x3456, false)
        );

        // 2. AbsoluteX: (Can cross)
        // Case A: No Cross
        cpu.write_u16(instruction_ptr + 2, 0x3400);
        cpu.x_register = 0x10;
        assert_eq!(
            cpu.get_operand_address(AddressingMode::AbsoluteX, instruction_ptr + 2),
            (0x3410, false)
        );
        // Case B: Page Cross (0x34FF + 1 = 0x3500)
        cpu.write_u16(instruction_ptr + 4, 0x34FF);
        cpu.x_register = 0x01;
        assert_eq!(
            cpu.get_operand_address(AddressingMode::AbsoluteX, instruction_ptr + 4),
            (0x3500, true)
        );

        // 3. AbsoluteY: (Can cross)
        // Case A: No Cross
        cpu.write_u16(instruction_ptr + 6, 0x3400);
        cpu.y_register = 0x10;
        assert_eq!(
            cpu.get_operand_address(AddressingMode::AbsoluteY, instruction_ptr + 6),
            (0x3410, false)
        );
        // Case B: Page Cross
        cpu.write_u16(instruction_ptr + 8, 0x34FF);
        cpu.y_register = 0x01;
        assert_eq!(
            cpu.get_operand_address(AddressingMode::AbsoluteY, instruction_ptr + 8),
            (0x3500, true)
        );

        // 4. Immediate: (Never crosses, returns address itself)
        assert_eq!(
            cpu.get_operand_address(AddressingMode::Immediate, instruction_ptr + 10),
            (instruction_ptr + 10, false)
        );

        // 5. Indirect: (JMP only, never has "page cross penalty" logic here)
        cpu.write_u16(instruction_ptr + 12, 0x1000); // Pointer location
        cpu.write_u16(0x1000, 0x5634); // Pointer value
        assert_eq!(
            cpu.get_operand_address(AddressingMode::Indirect, instruction_ptr + 12),
            (0x5634, false)
        );

        // 6. IndirectX: (Never crosses page boundary for penalty)
        cpu.write_u8(instruction_ptr + 14, 0x20); // Zero page addr
        cpu.x_register = 0x04;
        cpu.write_u16(0x24, 0x5634); // Value at $20+X
        assert_eq!(
            cpu.get_operand_address(AddressingMode::IndirectX, instruction_ptr + 14),
            (0x5634, false) // Always false for IndirectX
        );

        // 7. IndirectY: (Can cross)
        // Case A: No Cross
        cpu.write_u8(instruction_ptr + 16, 0x30); // Zero page addr
        cpu.write_u16(0x30, 0x1000); // Pointer points to 0x1000
        cpu.y_register = 0x10;
        assert_eq!(
            cpu.get_operand_address(AddressingMode::IndirectY, instruction_ptr + 16),
            (0x1010, false)
        );
        // Case B: Page Cross
        cpu.write_u8(instruction_ptr + 17, 0x32); // Zero page addr
        cpu.write_u16(0x32, 0x10FF); // Pointer points to 0x10FF
        cpu.y_register = 0x01;
        assert_eq!(
            cpu.get_operand_address(AddressingMode::IndirectY, instruction_ptr + 17),
            (0x1100, true) // <--- Expect True (0x10FF + 1 crosses to 0x11xx)
        );

        // 8. Relative: (Calculated in branch(), so this just returns target)
        cpu.write_u8(instruction_ptr + 18, 0x10);
        assert_eq!(
            cpu.get_operand_address(AddressingMode::Relative, instruction_ptr + 18),
            (instruction_ptr + 18, false)
        );

        // 9. ZeroPage Variants (Never cross)
        cpu.write_u8(instruction_ptr + 19, 0x42);
        assert_eq!(cpu.get_operand_address(AddressingMode::ZeroPage, instruction_ptr + 19), (0x0042, false));

        cpu.write_u8(instruction_ptr + 20, 0x42);
        cpu.x_register = 0x08;
        assert_eq!(cpu.get_operand_address(AddressingMode::ZeroPageX, instruction_ptr + 20), (0x004A, false));

        cpu.write_u8(instruction_ptr + 21, 0x42);
        cpu.y_register = 0x09;
        assert_eq!(cpu.get_operand_address(AddressingMode::ZeroPageY, instruction_ptr + 21), (0x004B, false));
    }

    #[test]
    fn test_get_operand_address_indirect_page_bug() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));

        // The Pointer LOCATION (The Instruction Operand)
        // We choose 0x0200, which is safe CPU RAM (0x0000-0x07FF).
        // We store the pointer address ($00FF) there.
        cpu.write_u8(0x0200, 0xFF);
        cpu.write_u8(0x0201, 0x00);

        // The Pointer VALUE (The Target)
        // Now we setup the data at $00FF so the bug can happen.
        // LSB at $00FF: 0x34
        cpu.write_u8(0x00FF, 0x34);

        // The Page Boundary Bug
        // MSB should be read from $0100, but due to bug, it wraps to $0000.
        cpu.write_u8(0x0000, 0x12); // Expected MSB
        cpu.write_u8(0x0100, 0x99); // "Correct" but ignored MSB

        // We pass 0x0200, where we stored the pointer ($00FF).
        let (target_address, _) = cpu.get_operand_address(AddressingMode::Indirect, 0x0200);

        assert_eq!(target_address, 0x1234, "Indirect addressing did not simulate page boundary bug correctly");
    }

    #[test]
    fn test_stack_push_pop_u8() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
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
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.push_u16(0x1234);
        assert_eq!(cpu.stack_pointer, 0xFD);
        let popped_value = cpu.pop_u16();
        assert_eq!(popped_value, 0x1234);
        assert_eq!(cpu.stack_pointer, 0xFF);
    }
}
