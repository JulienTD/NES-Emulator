use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleBit(& mut self, value: u8) -> u8 {
        // Perform bitwise AND between accumulator and memory operand
        let result = self.accumulator & value;

        // Set Zero flag (Z) - set if result == 0
        self.set_status_flag(StatusFlag::Zero, result == 0);

        // Set Overflow flag (V) - copy bit 6 of the memory operand
        self.set_status_flag(StatusFlag::Overflow, (value & 0x40) != 0);

        // Set Negative flag (N) - copy bit 7 of the memory operand
        self.set_status_flag(StatusFlag::Negative, (value & 0x80) != 0);

        return 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_bit_sets_zero_flag_when_and_zero() {
        let mut cpu = new_cpu();
        cpu.accumulator = 0xF0;
        // value has no overlapping bits with accumulator
        cpu.handleBit(0x0F);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        // V and N should reflect bits 6 and 7 of the operand
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
    }

    #[test]
    fn test_bit_sets_overflow_and_negative_from_operand() {
        let mut cpu = new_cpu();
        cpu.accumulator = 0xFF;
        // operand has bit 6 and bit 7 set
        cpu.handleBit(0xC0); // 0b1100_0000
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
    }

    #[test]
    fn test_bit_does_not_change_accumulator() {
        let mut cpu = new_cpu();
        cpu.accumulator = 0xAA;
        cpu.handleBit(0xFF);
        assert_eq!(cpu.accumulator, 0xAA);
    }
}
