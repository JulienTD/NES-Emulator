use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_dey(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let result = self.y_register.wrapping_sub(1);
        self.y_register = result;

        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, result & 0x80 != 0 );
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu6502::{new_cpu, StatusFlag};
    use crate::rom::Rom;

    #[test]
    fn test_dey_sets_flags_correctly() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));

        // Test result > 0
        cpu.y_register = 0x02;
        let extra = cpu.handle_dey(None, None);
        assert_eq!(extra, 0);
        assert_eq!(cpu.y_register, 0x01);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test result == 0
        cpu.y_register = 0x01;
        let extra = cpu.handle_dey(None, None);
        assert_eq!(extra, 0);
        assert_eq!(cpu.y_register, 0x00);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test result < 0
        cpu.y_register = 0x00;
        let extra = cpu.handle_dey(None, None);
        assert_eq!(extra, 0);
        assert_eq!(cpu.y_register, 0xFF);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
    }

    #[test]
    fn test_dey_zero_flag_set_when_result_is_zero() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.y_register = 0x01;
        cpu.handle_dey(None, None);
        assert_eq!(cpu.y_register, 0x00);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
    }

    #[test]
    fn test_dey_negative_flag_set_when_bit7_set() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // Wrap-around: 0x00 - 1 = 0xFF (bit 7 set)
        cpu.y_register = 0x00;
        cpu.handle_dey(None, None);
        assert_eq!(cpu.y_register, 0xFF);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
    }

    #[test]
    fn test_dey_wraps_around_from_zero_to_0xff() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.y_register = 0x00;
        cpu.handle_dey(None, None);
        assert_eq!(cpu.y_register, 0xFF);
    }

    #[test]
    fn test_dey_does_not_affect_other_registers() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.y_register = 0x05;
        cpu.accumulator = 0xAB;
        cpu.x_register = 0xCD;
        cpu.handle_dey(None, None);
        assert_eq!(cpu.accumulator, 0xAB);
        assert_eq!(cpu.x_register, 0xCD);
    }
}
