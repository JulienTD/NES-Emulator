use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleDEY(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let result = self.y_register.wrapping_sub(1);

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
    fn test_dey_sets_flags_correctly() {
        let mut cpu = crate::cpu6502::new_cpu();

        // Test result > 0
        cpu.y_register = 0x02;
        let extra = cpu.handleDEY(None, None);
        assert_eq!(extra, 0);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test result == 0
        cpu.y_register = 0x01;
        let extra = cpu.handleDEY(None, None);
        assert_eq!(extra, 0);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test result < 0
        cpu.y_register = 0x00;
        let extra = cpu.handleDEY(None, None);
        assert_eq!(extra, 0);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
    }
}
