use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_pla(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = self.pop_u8();
        self.accumulator = value;

        self.set_status_flag(StatusFlag::Zero, self.accumulator == 0);
        self.set_status_flag(StatusFlag::Negative, (self.accumulator & 0x80) != 0);

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu6502::{new_cpu, StatusFlag};
    use crate::rom::Rom;

    #[test]
    fn test_pla_pulls_value_and_sets_flags() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // Manually push a value to the stack to be pulled
        cpu.push_u8(0x42);
        assert_eq!(cpu.stack_pointer, 0xFE);

        cpu.handle_pla(None, None);

        assert_eq!(cpu.accumulator, 0x42);
        assert_eq!(cpu.stack_pointer, 0xFF, "Stack pointer should increment");
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_pla_sets_zero_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.push_u8(0x00);
        cpu.handle_pla(None, None);
        assert_eq!(cpu.accumulator, 0x00);
        assert!(cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_pla_sets_negative_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.push_u8(0x80);
        cpu.handle_pla(None, None);
        assert_eq!(cpu.accumulator, 0x80);
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert!(cpu.get_status_flag(StatusFlag::Negative));
    }
}
