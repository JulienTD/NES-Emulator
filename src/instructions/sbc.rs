use crate::cpu6502::{CPU, StatusFlag};
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handle_sbc(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of SBC should be present");

        // SBC is implemented as ADC with the operand's bits inverted.
        // A - M - (1-C) is equivalent to A + !M + C
        let inverted_value = !value;

        // Get current carry flag and operands
        let carry_in = if self.get_status_flag(StatusFlag::Carry) { 1 } else { 0 };

        // Perform addition
        let sum = (self.accumulator as u16) + (inverted_value as u16) + carry_in;
        let result = sum as u8;

        // Set Carry flag (C) - set if sum > 255
        self.set_status_flag(StatusFlag::Carry, sum > 0xFF);

        // Set Zero flag (Z) - set if result = 0
        self.set_status_flag(StatusFlag::Zero, result == 0);

        // Set Negative flag (N) - set if bit 7 of result is set
        self.set_status_flag(StatusFlag::Negative, (result & 0x80) != 0);

        // Set Overflow flag (V) - using signed arithmetic
        // Convert to signed integers for comparison
        let signed_accumulator = self.accumulator as i8;
        let signed_value = inverted_value as i8;
        let signed_result = result as i8;

        // Overflow occurs if:
        // 1. Adding two positive numbers results in a negative number, or
        // 2. Adding two negative numbers results in a positive number
        let overflow = (signed_accumulator >= 0 && signed_value >= 0 && signed_result < 0) ||
                       (signed_accumulator < 0 && signed_value < 0 && signed_result >= 0);
        self.set_status_flag(StatusFlag::Overflow, overflow);

        self.accumulator = result;
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_sbc_basic_subtraction() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x10;
        cpu.set_status_flag(StatusFlag::Carry, true); // No borrow
        cpu.handle_sbc(Some(0x05), None);
        assert_eq!(cpu.accumulator, 0x0B); // 16 - 5 = 11
        assert!(cpu.get_status_flag(StatusFlag::Carry)); // No borrow occurred
        assert!(!cpu.get_status_flag(StatusFlag::Overflow));
    }

    #[test]
    fn test_sbc_with_borrow() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x10;
        cpu.set_status_flag(StatusFlag::Carry, false); // With borrow
        cpu.handle_sbc(Some(0x05), None);
        assert_eq!(cpu.accumulator, 0x0A); // 16 - 5 - 1 = 10
        assert!(cpu.get_status_flag(StatusFlag::Carry)); // No borrow occurred
    }

    #[test]
    fn test_sbc_causes_borrow_and_overflow() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x80; // -128
        cpu.set_status_flag(StatusFlag::Carry, true); // No borrow
        cpu.handle_sbc(Some(0x01), None); // -128 - 1 = -129 (overflows to +127)
        assert_eq!(cpu.accumulator, 0x7F);
        assert!(cpu.get_status_flag(StatusFlag::Carry), "No borrow should occur");
        assert!(cpu.get_status_flag(StatusFlag::Overflow), "Overflow should be set");
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
    }
}
