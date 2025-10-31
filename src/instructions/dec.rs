use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleDEC(& mut self, opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of DEC should be present");
        let result = value.wrapping_sub(1);

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
    fn test_dec_sets_flags_correctly() {
        let mut cpu = crate::cpu6502::new_cpu();

        // Test result > 0
        let extra = cpu.handleDEC(Some(0x02), None);
        assert_eq!(extra, 0);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test result == 0
        let extra = cpu.handleDEC(Some(0x01), None);
        assert_eq!(extra, 0);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test result < 0
        let extra = cpu.handleDEC(Some(0x00), None);
        assert_eq!(extra, 0);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
    }
}
