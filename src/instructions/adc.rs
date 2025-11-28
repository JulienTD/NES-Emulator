use crate::cpu6502::{CPU, StatusFlag};
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handle_adc(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of ADC should be present");

        // Get current carry flag and operands
        let carry_in = if self.get_status_flag(StatusFlag::Carry) { 1 } else { 0 };

        // Perform addition
        let sum = (self.accumulator as u16) + (value as u16) + carry_in;
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
        let signed_value = value as i8;
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
    fn test_adc_instruction() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x14;
        cpu.handle_adc(Some(0x27), None);
        assert_eq!(cpu.accumulator, 0x3B);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), false);
    }

    #[test]
    fn test_adc_with_carry() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0xFF;
        cpu.set_status_flag(StatusFlag::Carry, true);
        cpu.handle_adc(Some(0x01), None);
        assert_eq!(cpu.accumulator, 0x01);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), false);
    }

    #[test]
    fn test_adc_overflow() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x7F;
        cpu.handle_adc(Some(0x01), None);
        assert_eq!(cpu.accumulator, 0x80);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), true);
    }

    #[test]
    fn test_adc_zero_result() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x00;
        cpu.handle_adc(Some(0x00), None);
        assert_eq!(cpu.accumulator, 0x00);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), false);
    }

    #[test]
    fn test_adc_negative_result() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x80;
        cpu.handle_adc(Some(0x00), None);
        assert_eq!(cpu.accumulator, 0x80);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), false);
    }

    #[test]
    fn test_adc_with_carry_in() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x50;
        cpu.set_status_flag(StatusFlag::Carry, true);
        cpu.handle_adc(Some(0x30), None);
        assert_eq!(cpu.accumulator, 0x81);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), true);
    }

    #[test]
    fn test_adc_max_values() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0xFF;
        cpu.handle_adc(Some(0xFF), None);
        assert_eq!(cpu.accumulator, 0xFE);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), false);
    }

    #[test]
    fn test_adc_min_values() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x00;
        cpu.handle_adc(Some(0x00), None);
        assert_eq!(cpu.accumulator, 0x00);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), false);
    }

    #[test]
    fn test_adc_with_carry_and_overflow() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x7F;
        cpu.set_status_flag(StatusFlag::Carry, true);
        cpu.handle_adc(Some(0x01), None);
        assert_eq!(cpu.accumulator, 0x81);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), true);
    }

    #[test]
    fn test_adc_resulting_in_zero_with_carry() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0xFF;
        cpu.set_status_flag(StatusFlag::Carry, true);
        cpu.handle_adc(Some(0x00), None);
        assert_eq!(cpu.accumulator, 0x00);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), false);
    }

    #[test]
    fn test_adc_large_value() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x10;
        cpu.handle_adc(Some(0xF0), None);
        assert_eq!(cpu.accumulator, 0x00);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), false);
    }

    #[test]
    fn test_adc_no_flags_set() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x20;
        cpu.handle_adc(Some(0x10), None);
        assert_eq!(cpu.accumulator, 0x30);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), false);
    }

    #[test]
    fn test_adc_all_flags_set() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x7F;
        cpu.handle_adc(Some(0x80), None);
        assert_eq!(cpu.accumulator, 0xFF);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), false);
    }

    #[test]
    fn test_adc_with_carry_and_zero() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0xFF;
        cpu.set_status_flag(StatusFlag::Carry, true);
        cpu.handle_adc(Some(0x00), None);
        assert_eq!(cpu.accumulator, 0x00);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), false);
    }

    #[test]
    fn test_adc_with_negative_result_and_carry() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x80;
        cpu.set_status_flag(StatusFlag::Carry, true);
        cpu.handle_adc(Some(0x7F), None);
        assert_eq!(cpu.accumulator, 0x00);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), false);
    }

    #[test]
    fn test_adc_with_overflow_and_negative() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x40;
        cpu.handle_adc(Some(0x40), None);
        assert_eq!(cpu.accumulator, 0x80);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), true);
    }

    #[test]
    fn test_adc_with_carry_and_overflow_flags() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0xFF;
        cpu.set_status_flag(StatusFlag::Carry, true);
        cpu.handle_adc(Some(0x02), None);
        assert_eq!(cpu.accumulator, 0x02);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Overflow), false);
    }
}
