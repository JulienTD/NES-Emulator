use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleCPY(& mut self, opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of CPY should be present");
        let result = self.y_register.wrapping_sub(value);

        // The status of the flags after comparison can be determined as follows:
        // Carry Flag (C): Set if Y >= M
        // Zero Flag (Z): Set if Y == M
        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, result & 0x80 != 0 );
        self.set_status_flag(StatusFlag::Carry, self.y_register >= value);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_cpy_sets_flags_correctly() {
        let mut cpu = crate::cpu6502::new_cpu();
        cpu.y_register = 0x50;

        // Test Y > M
        cpu.handleCPY(Some(0x30), None);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test Y == M
        cpu.handleCPY(Some(0x50), None);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test Y < M
        cpu.handleCPY(Some(0x70), None);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
    }
}
