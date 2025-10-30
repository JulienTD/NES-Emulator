use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleCPX(& mut self, value: u8) -> u8 {
        let result = self.x_register.wrapping_sub(value);

        // The status of the flags after comparison can be determined as follows:
        // Carry Flag (C): Set if A >= M
        // Zero Flag (Z): Set if A == M
        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, result & 0x80 != 0 );
        self.set_status_flag(StatusFlag::Carry, self.x_register >= value);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_cpx_sets_flags_correctly() {
        let mut cpu = crate::cpu6502::new_cpu();
        cpu.x_register = 0x50;

        // Test X > M
        cpu.handleCPX(0x30);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test X == M
        cpu.handleCPX(0x50);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test X < M
        cpu.handleCPX(0x70);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
    }
}
