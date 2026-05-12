use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_dec(& mut self, opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of DEC should be present");
        let address = opt_address.expect("BUG: address of DEC should be present");

        let result = value.wrapping_sub(1);
        self.write_u8(address, result);

        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, result & 0x80 != 0 );
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu6502::{new_cpu, StatusFlag};
    use crate::rom::Rom;

    #[test]
    fn test_dec_sets_flags_correctly() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0010;

        // Test result > 0
        let extra = cpu.handle_dec(Some(0x02), Some(addr));
        assert_eq!(extra, 0);
        assert_eq!(cpu.read_u8(addr), 0x01);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
        // Check write back
        assert_eq!(cpu.read_u8(addr), 0x01);

        // Test result == 0
        let extra = cpu.handle_dec(Some(0x01), Some(addr));
        assert_eq!(extra, 0);
        assert_eq!(cpu.read_u8(addr), 0x00);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
        assert_eq!(cpu.read_u8(addr), 0x00);

        // Test result < 0
        let extra = cpu.handle_dec(Some(0x00), Some(addr));
        assert_eq!(extra, 0);
        assert_eq!(cpu.read_u8(addr), 0xFF);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
        assert_eq!(cpu.read_u8(addr), 0xFF);
    }

    #[test]
    fn test_dec_zero_flag_set_when_result_is_zero() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0020;
        cpu.handle_dec(Some(0x01), Some(addr));
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);
        assert_eq!(cpu.read_u8(addr), 0x00);
    }

    #[test]
    fn test_dec_negative_flag_set_when_result_has_bit7() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0020;
        // 0x80 - 1 = 0x7F: bit 7 NOT set; use 0x00 - 1 = 0xFF (bit 7 set)
        cpu.handle_dec(Some(0x00), Some(addr));
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
        assert_eq!(cpu.read_u8(addr), 0xFF);
    }

    #[test]
    fn test_dec_wraps_around_from_zero_to_0xff() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0030;
        cpu.handle_dec(Some(0x00), Some(addr));
        assert_eq!(cpu.read_u8(addr), 0xFF);
    }

    #[test]
    fn test_dec_writes_result_to_correct_address() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0040;
        let other_addr = 0x0050;
        cpu.handle_dec(Some(0x05), Some(addr));
        assert_eq!(cpu.read_u8(addr), 0x04);
        // The other address should be untouched (default 0)
        assert_eq!(cpu.read_u8(other_addr), 0x00);
    }
}
