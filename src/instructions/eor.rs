use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleEOR(& mut self, value: u8) -> u8 {
        let result = self.accumulator ^ value;
        self.accumulator = result;
        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, result & 0x80 != 0 );
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_eor_sets_flags_correctly() {
        let mut cpu = crate::cpu6502::new_cpu();

        // Test result > 0
        cpu.accumulator = 0b10101010;
        let extra = cpu.handleEOR(0b01010101);
        assert_eq!(extra, 0);
        assert_eq!(cpu.accumulator, 0b11111111);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);

        // Test result == 0
        cpu.accumulator = 0b11110000;
        let extra = cpu.handleEOR(0b11110000);
        assert_eq!(extra, 0);
        assert_eq!(cpu.accumulator, 0b00000000);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test result < 0
        cpu.accumulator = 0b00001111;
        let extra = cpu.handleEOR(0b11110000);
        assert_eq!(extra, 0);
        assert_eq!(cpu.accumulator, 0b11111111);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
    }
}
