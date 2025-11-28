use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_tsx(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        self.x_register = self.stack_pointer;

        self.set_status_flag(StatusFlag::Zero, self.x_register == 0);
        self.set_status_flag(StatusFlag::Negative, (self.x_register & 0x80) != 0);

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu6502::{new_cpu, StatusFlag};
    use crate::rom::Rom;

    #[test]
    fn test_tsx_transfers_value_and_sets_flags() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.stack_pointer = 0x42;
        cpu.handle_tsx(None, None);
        assert_eq!(cpu.x_register, 0x42);
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_tsx_sets_zero_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.stack_pointer = 0x00;
        cpu.handle_tsx(None, None);
        assert_eq!(cpu.x_register, 0x00);
        assert!(cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_tsx_sets_negative_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.stack_pointer = 0x80;
        cpu.handle_tsx(None, None);
        assert_eq!(cpu.x_register, 0x80);
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert!(cpu.get_status_flag(StatusFlag::Negative));
    }
}
