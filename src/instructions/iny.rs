use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_iny(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let result = self.y_register.wrapping_add(1);
        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, result & 0x80 != 0);
        self.y_register = result;
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu6502::{new_cpu, StatusFlag};
    use crate::rom::Rom;

    #[test]
    fn test_iny_increments_x_register() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.y_register = 0x10;
        cpu.handle_iny(None, None);
        assert_eq!(cpu.y_register, 0x11);
    }

    #[test]
    fn test_iny_sets_zero_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.y_register = 0xFF;
        cpu.handle_iny(None, None);
        assert!(cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
        assert_eq!(cpu.y_register, 0x00);
    }

    #[test]
    fn test_iny_sets_negative_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.y_register = 0x7F;
        cpu.handle_iny(None, None);
        assert!(cpu.get_status_flag(StatusFlag::Negative));
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert_eq!(cpu.y_register, 0x80);
    }
}
