use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_ora(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of ORA should be present");

        self.accumulator |= value;
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
    fn test_ora_sets_accumulator() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0b0000_1100;
        cpu.handle_ora(Some(0b0000_0011), None);
        assert_eq!(cpu.accumulator, 0b0000_1111);
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
    }
    #[test]
    fn test_ora_sets_zero_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0b0000_0000;
        cpu.handle_ora(Some(0b0000_0000), None);
        assert_eq!(cpu.accumulator, 0b0000_0000);
        assert!(cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
    }
    #[test]
    fn test_ora_sets_negative_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0b0000_0001;
        cpu.handle_ora(Some(0b1000_0000), None);
        assert_eq!(cpu.accumulator, 0b1000_0001);
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert!(cpu.get_status_flag(StatusFlag::Negative));
    }
}
