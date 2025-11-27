use crate::cpu6502::{CPU, StatusFlag};
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handleINX(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let result = self.x_register.wrapping_add(1);
        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, result & 0x80 != 0);
        self.x_register = result;
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;
    #[test]
    fn test_inx_increments_x_register() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.x_register = 0x10;
        cpu.handleINX(None, None);
        assert_eq!(cpu.x_register, 0x11);
    }
    #[test]
    fn test_inx_sets_zero_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.x_register = 0xFF;
        cpu.handleINX(None, None);
        assert!(cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
        assert_eq!(cpu.x_register, 0x00);
    }
    #[test]
    fn test_inx_sets_negative_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.x_register = 0x7F;
        cpu.handleINX(None, None);
        assert!(cpu.get_status_flag(StatusFlag::Negative));
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert_eq!(cpu.x_register, 0x80);
    }
}
