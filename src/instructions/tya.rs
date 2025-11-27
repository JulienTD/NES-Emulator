use crate::cpu6502::{CPU, StatusFlag};
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handleTYA(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        self.accumulator = self.y_register;

        self.set_status_flag(StatusFlag::Zero, self.accumulator == 0);
        self.set_status_flag(StatusFlag::Negative, (self.accumulator & 0x80) != 0);

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_tya_transfers_value_and_sets_flags() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.y_register = 0x42;
        cpu.handleTYA(None, None);
        assert_eq!(cpu.accumulator, 0x42);
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_tya_sets_zero_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.y_register = 0x00;
        cpu.handleTYA(None, None);
        assert_eq!(cpu.accumulator, 0x00);
        assert!(cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_tya_sets_negative_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.y_register = 0x80;
        cpu.handleTYA(None, None);
        assert_eq!(cpu.accumulator, 0x80);
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert!(cpu.get_status_flag(StatusFlag::Negative));
    }
}
