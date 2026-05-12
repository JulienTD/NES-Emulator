use crate::cpu6502::CPU;

impl CPU {
    pub(crate) fn handle_sty(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let address = _opt_address.expect("BUG: address of STY should be present");
        self.write_u8(address, self.y_register);
        return 0;
    }
}

#[cfg(test)]
mod tests {

    use crate::bus::Bus;
    use crate::cpu6502::new_cpu;
    use crate::rom::Rom;

    #[test]
    fn test_sty_stores_y_register_in_memory() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let address = 0x0200;
        cpu.y_register = 0x42;
        let initial_status = cpu.status_register;

        let cycles = cpu.handle_sty(None, Some(address));

        assert_eq!(cycles, 0, "STY should not return extra cycles");
        assert_eq!(cpu.read_u8(address), 0x42, "Y register value should be stored at the address");
        assert_eq!(cpu.status_register, initial_status, "STY should not affect any flags");
    }

    #[test]
    fn test_sty_stores_zero_value() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let address = 0x0010;
        cpu.y_register = 0x00;
        cpu.handle_sty(None, Some(address));
        assert_eq!(cpu.read_u8(address), 0x00);
    }

    #[test]
    fn test_sty_stores_0xff_value() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let address = 0x0010;
        cpu.y_register = 0xFF;
        cpu.handle_sty(None, Some(address));
        assert_eq!(cpu.read_u8(address), 0xFF);
    }

    #[test]
    fn test_sty_stores_to_different_addresses() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.y_register = 0x11;
        cpu.handle_sty(None, Some(0x0010));
        cpu.y_register = 0x22;
        cpu.handle_sty(None, Some(0x0020));
        assert_eq!(cpu.read_u8(0x0010), 0x11);
        assert_eq!(cpu.read_u8(0x0020), 0x22);
    }

    #[test]
    fn test_sty_overwrites_existing_memory() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let address = 0x0010;
        cpu.y_register = 0xAA;
        cpu.handle_sty(None, Some(address));
        cpu.y_register = 0xBB;
        cpu.handle_sty(None, Some(address));
        assert_eq!(cpu.read_u8(address), 0xBB);
    }
}
