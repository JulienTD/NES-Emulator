use crate::cpu6502::CPU;

impl CPU {
    // SYA (SHY/SAY) - AND Y register with the high byte of the argument + 1, store result into memory
    // M = Y & (HIGH(arg) + 1)
    // No flags affected.
    pub(crate) fn handle_sya(& mut self, _opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let address = opt_address.expect("BUG: address of SYA should be present");

        let high = (address >> 8) as u8;
        let result = self.y_register & high.wrapping_add(1);
        self.write_u8(address, result);

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu6502::new_cpu;
    use crate::bus::Bus;
    use crate::rom::Rom;

    #[test]
    fn test_sya_stores_y_and_high_plus_one() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.y_register = 0x0F;

        let addr: u16 = 0x0302; // high=0x03 -> high+1=0x04
        cpu.write_u8(addr, 0x00);

        let _ = cpu.handle_sya(None, Some(addr));

        // expected = 0x0F & 0x04 = 0x04
        assert_eq!(cpu.read_u8(addr), 0x04);
    }

    #[test]
    fn test_sya_high_plus_one_wrap_behavior() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.y_register = 0xFF;

        // choose a writable address with high byte 0x01 -> high+1 = 0x02
        let addr: u16 = 0x0166;
        cpu.write_u8(addr, 0xFF);

        let _ = cpu.handle_sya(None, Some(addr));

        // result = 0xFF & 0x02 = 0x02
        assert_eq!(cpu.read_u8(addr), 0x02);
    }
}
