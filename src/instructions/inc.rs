use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleINC(& mut self, opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of INC should be present");
        let address = opt_address.expect("BUG: address of INC should be present");

        let value = value.wrapping_add(1);
        self.write_u8(address, value);
        self.set_status_flag(StatusFlag::Zero, value == 0);
        self.set_status_flag(StatusFlag::Negative, (value & 0x80) != 0);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_inc_increments_value() {
        let mut cpu = new_cpu();
        let address = 0x2000;
        cpu.write_u8(address, 0x05);

        let extra = cpu.handleINC(Some(0x05), Some(address));
        let result = cpu.read_u8(address);

        assert_eq!(result, 0x06);
        assert_eq!(extra, 0);
    }

    #[test]
    fn test_inc_wraps_around() {
        let mut cpu = new_cpu();
        let address = 0x2000;
        cpu.write_u8(address, 0xFF);

        let extra = cpu.handleINC(Some(0xFF), Some(address));
        let result = cpu.read_u8(address);

        assert_eq!(result, 0x00);
        assert_eq!(extra, 0);
    }

    #[test]
    fn test_inc_sets_flags_correctly() {
        let mut cpu = new_cpu();
        let address = 0x2000;

        // Test result > 0
        cpu.write_u8(address, 0x05);
        let _extra = cpu.handleINC(Some(0x05), Some(address));
        let result = cpu.read_u8(address);
        assert_eq!(result, 0x06);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test result == 0
        cpu.write_u8(address, 0xFF);
        let _extra = cpu.handleINC(Some(0xFF), Some(address));
        let result = cpu.read_u8(address);
        assert_eq!(result, 0x00);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test result < 0
        cpu.write_u8(address, 0x7F);
        let _extra = cpu.handleINC(Some(0x7F), Some(address));
        let result = cpu.read_u8(address);
        assert_eq!(result, 0x80);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
    }
}