use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleAND(& mut self, value: u8) -> u8 {
        let result = self.accumulator & value;

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
    // AND Instruction Tests
    #[test]
    fn test_and_instruction() {
        let mut cpu = new_cpu();
        cpu.accumulator = 0xF0;
        cpu.handleAND(0x0F);
        assert_eq!(cpu.accumulator, 0x00);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
    }

    #[test]
    fn test_and_negative_result() {
        let mut cpu = new_cpu();
        cpu.accumulator = 0xFF;
        cpu.handleAND(0x80);
        assert_eq!(cpu.accumulator, 0x80);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
    }

    #[test]
    fn test_and_no_flags_set() {
        let mut cpu = new_cpu();
        cpu.accumulator = 0x7F;
        cpu.handleAND(0x3F);
        assert_eq!(cpu.accumulator, 0x3F);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
    }
}
