use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_cpx(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of CPX should be present");
        let result = self.x_register.wrapping_sub(value);

        // The status of the flags after comparison can be determined as follows:
        // Carry Flag (C): Set if X >= M
        // Zero Flag (Z): Set if X == M
        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, result & 0x80 != 0 );
        self.set_status_flag(StatusFlag::Carry, self.x_register >= value);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu6502::{new_cpu, StatusFlag};
    use crate::rom::Rom;

    #[test]
    fn test_cpx_sets_flags_correctly() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.x_register = 0x50;

        // Test X > M
        cpu.handle_cpx(Some(0x30), None);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test X == M
        cpu.handle_cpx(Some(0x50), None);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test X < M
        cpu.handle_cpx(Some(0x70), None);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
    }
}
