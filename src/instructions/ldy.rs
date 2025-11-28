use crate::cpu6502::{CPU, StatusFlag};
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handle_ldy(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of LDA should be present");
        self.y_register = value;

        self.set_status_flag(StatusFlag::Zero, self.y_register == 0);
        self.set_status_flag(StatusFlag::Negative, (self.y_register & 0x80) != 0);

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_lda_load_value() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.handle_ldy(Some(0x42), None);
        assert_eq!(cpu.y_register, 0x42);
        assert!(!cpu.get_status_flag(StatusFlag::Zero), "Zero flag should be clear");
        assert!(!cpu.get_status_flag(StatusFlag::Negative), "Negative flag should be clear");
    }

    #[test]
    fn test_lda_sets_zero_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.handle_ldy(Some(0x00), None);
        assert_eq!(cpu.y_register, 0x00);
        assert!(cpu.get_status_flag(StatusFlag::Zero), "Zero flag should be set");
        assert!(!cpu.get_status_flag(StatusFlag::Negative), "Negative flag should be clear");
    }

    #[test]
    fn test_lda_sets_negative_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.handle_ldy(Some(0x80), None);
        assert_eq!(cpu.y_register, 0x80);
        assert!(!cpu.get_status_flag(StatusFlag::Zero), "Zero flag should be clear");
        assert!(cpu.get_status_flag(StatusFlag::Negative), "Negative flag should be set");
    }
}
