use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleLDX(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of LDX should be present");
        self.x_register = value;

        self.set_status_flag(StatusFlag::Zero, self.x_register == 0);
        self.set_status_flag(StatusFlag::Negative, (self.x_register & 0x80) != 0);

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_ldx_load_value() {
        let mut cpu = new_cpu();
        cpu.handleLDX(Some(0x42), None);
        assert_eq!(cpu.x_register, 0x42);
        assert!(!cpu.get_status_flag(StatusFlag::Zero), "Zero flag should be clear");
        assert!(!cpu.get_status_flag(StatusFlag::Negative), "Negative flag should be clear");
    }

    #[test]
    fn test_ldx_sets_zero_flag() {
        let mut cpu = new_cpu();
        cpu.handleLDX(Some(0x00), None);
        assert_eq!(cpu.x_register, 0x00);
        assert!(cpu.get_status_flag(StatusFlag::Zero), "Zero flag should be set");
        assert!(!cpu.get_status_flag(StatusFlag::Negative), "Negative flag should be clear");
    }

    #[test]
    fn test_ldx_sets_negative_flag() {
        let mut cpu = new_cpu();
        cpu.handleLDX(Some(0x80), None);
        assert_eq!(cpu.x_register, 0x80);
        assert!(!cpu.get_status_flag(StatusFlag::Zero), "Zero flag should be clear");
        assert!(cpu.get_status_flag(StatusFlag::Negative), "Negative flag should be set");
    }
}
