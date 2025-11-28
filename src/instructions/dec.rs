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
}
