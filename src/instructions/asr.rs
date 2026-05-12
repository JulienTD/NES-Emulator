use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_asr(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of ASR should be present");
        let temp = self.accumulator & value;

        // Set carry from bit0 before shift
        self.set_status_flag(StatusFlag::Carry, (temp & 0x01) != 0);

        // Shift right
        let result = temp >> 1;
        self.accumulator = result;

        // Set Zero and Negative
        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, (result & 0x80) != 0);

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu6502::new_cpu;
    use crate::bus::Bus;
    use crate::rom::Rom;
    use crate::cpu6502::StatusFlag;

    #[test]
    fn test_asr_and_then_lsr() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0b0000_0011;
        let _ = cpu.handle_asr(Some(0b0000_0011), None);
        // temp = 3, shift => 1
        assert_eq!(cpu.accumulator, 0b0000_0001);
        assert!(cpu.get_status_flag(StatusFlag::Carry));
    }

    #[test]
    fn test_asr_zero_flag_when_result_is_zero() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // AND result is 0 -> shift produces 0
        cpu.accumulator = 0xAA; // 0b1010_1010
        let _ = cpu.handle_asr(Some(0x55), None); // 0b0101_0101 -> AND = 0, no carry
        assert_eq!(cpu.accumulator, 0x00);
        assert!(cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Carry));
    }

    #[test]
    fn test_asr_carry_cleared_when_bit0_of_and_result_is_zero() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // AND result with bit0=0 => carry cleared, result shifts to 0 from 0x02
        cpu.accumulator = 0xFF;
        let _ = cpu.handle_asr(Some(0x02), None); // temp = 0x02 (bit0=0), result = 0x01
        assert_eq!(cpu.accumulator, 0x01);
        assert!(!cpu.get_status_flag(StatusFlag::Carry));
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
    }

    #[test]
    fn test_asr_negative_flag_never_set_after_right_shift() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // LSR always puts 0 into bit7, so Negative should never be set
        cpu.accumulator = 0xFF;
        let _ = cpu.handle_asr(Some(0xFF), None); // temp=0xFF, result=0x7F
        assert_eq!(cpu.accumulator, 0x7F);
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
    }
}
