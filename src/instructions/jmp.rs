use crate::cpu6502::CPU;
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handleJMP(& mut self, _opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let address = opt_address.expect("BUG: address of JMP should be present");
        self.program_counter = address;
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::{AddressingMode, new_cpu};

    #[test]
    fn test_jmp_sets_program_counter() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.handleJMP( None, Some(0x1234));
        assert_eq!(cpu.program_counter, 0x1234);
    }

    #[test]
    fn test_jmp_indirect_with_page_boundary_bug() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.program_counter = 0x8000;

        // The indirect vector is at a page boundary: 0x10FF
        cpu.write_u16(0x8000, 0x10FF);

        // The target address is 0xABCD.
        // LSB (0xCD) is at 0x10FF.
        cpu.write_u8(0x10FF, 0xCD);
        // MSB (0xAB) should be at 0x1100, but due to the bug, it's read from 0x1000.
        cpu.write_u8(0x1000, 0xAB);

        let target_address = cpu.get_operand_address(AddressingMode::Indirect, 0x8000);
        assert_eq!(target_address, (0xABCD, false), "The emulated 6502 indirect JMP bug should be present");
    }
}
