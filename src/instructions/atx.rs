use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    // ATX: AND immediate with accumulator, then transfer accumulator to X
    pub(crate) fn handle_atx(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of ATX should be present");
        self.accumulator = self.accumulator & value;
        self.x_register = self.accumulator;

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
    fn test_atx_and_transfers_to_x() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0b1010_1010;
        let _ = cpu.handle_atx(Some(0b1100_1100), None);
        assert_eq!(cpu.accumulator, 0b1000_1000);
        assert_eq!(cpu.x_register, 0b1000_1000);
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert!(cpu.get_status_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_atx_zero_flag_when_and_result_is_zero() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0xF0;
        let _ = cpu.handle_atx(Some(0x0F), None); // 0xF0 & 0x0F == 0
        assert_eq!(cpu.accumulator, 0x00);
        assert_eq!(cpu.x_register, 0x00);
        assert!(cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_atx_both_a_and_x_receive_same_result() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x3C;
        let _ = cpu.handle_atx(Some(0xFF), None); // result == 0x3C
        assert_eq!(cpu.accumulator, 0x3C);
        assert_eq!(cpu.x_register, 0x3C);
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
    }
}
