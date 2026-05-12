use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_cmp(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of CMP should be present");
        let result = self.accumulator.wrapping_sub(value);

        // The status of the flags after comparison can be determined as follows:
        // Carry Flag (C): Set if A >= M
        // Zero Flag (Z): Set if A == M
        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, result & 0x80 != 0 );
        self.set_status_flag(StatusFlag::Carry, self.accumulator >= value);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu6502::{new_cpu, StatusFlag};
    use crate::rom::Rom;

    #[test]
    fn test_cmp_sets_flags_correctly() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x50;

        // Test A > M
        cpu.handle_cmp(Some(0x30), None);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test A == M
        cpu.handle_cmp(Some(0x50), None);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test A < M
        cpu.handle_cmp(Some(0x70), None);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
    }

    #[test]
    fn test_cmp_zero_flag_set_when_equal() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x00;
        cpu.handle_cmp(Some(0x00), None);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
    }

    #[test]
    fn test_cmp_negative_flag_set_when_result_has_bit7() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // A=0x10, M=0x20 => result = 0xF0 (bit 7 set)
        cpu.accumulator = 0x10;
        cpu.handle_cmp(Some(0x20), None);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
    }

    #[test]
    fn test_cmp_carry_set_when_a_greater_than_m() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0xFF;
        cpu.handle_cmp(Some(0x01), None);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
    }

    #[test]
    fn test_cmp_carry_cleared_when_a_less_than_m() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x00;
        cpu.handle_cmp(Some(0x01), None);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), false);
    }

    #[test]
    fn test_cmp_returns_zero_extra_cycles() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x42;
        let extra = cpu.handle_cmp(Some(0x42), None);
        assert_eq!(extra, 0);
    }
}
