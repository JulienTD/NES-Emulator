use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleASL(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of ASL should be present");
        let result = value << 1;

        // Set Carry flag (C) - set if bit 7 of original value is set
        self.set_status_flag(StatusFlag::Carry, (value & 0x80) != 0);

        // Set Zero flag (Z) - set if result = 0
        self.set_status_flag(StatusFlag::Zero, result == 0);

        // Set Negative flag (N) - set if bit 7 of result is set
        self.set_status_flag(StatusFlag::Negative, (result & 0x80) != 0);

        self.accumulator = result;
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;
    // ASL Instruction Tests
    #[test]
    fn test_asl_instruction() {
        let mut cpu = new_cpu();
        cpu.accumulator = 0x40;
        cpu.handleASL(Some(0x40), None);
        assert_eq!(cpu.accumulator, 0x80);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
    }

    #[test]
    fn test_asl_sets_carry_flag() {
        let mut cpu = new_cpu();
        cpu.accumulator = 0x80;
        cpu.handleASL(Some(0x80), None);
        assert_eq!(cpu.accumulator, 0x00);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
    }
}
