use crate::cpu6502::CPU;

impl CPU {
    pub(crate) fn handle_aax(& mut self, _opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let address = opt_address.expect("BUG: address for AAX should be present");
        let value = self.accumulator & self.x_register;
        self.write_u8(address, value);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu6502::{new_cpu};
    use crate::rom::Rom;

    #[test]
    fn test_aax_stores_and_of_a_and_x_in_memory() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0xF0;
        cpu.x_register = 0x0F;
        let addr = 0x0200;

        let cycles = cpu.handle_aax(None, Some(addr));
        assert_eq!(cycles, 0);
        assert_eq!(cpu.read_u8(addr), 0x00); // 0xF0 & 0x0F == 0x00

        cpu.accumulator = 0xAB;
        cpu.x_register = 0x0B;
        let _ = cpu.handle_aax(None, Some(addr));
        assert_eq!(cpu.read_u8(addr), 0x0B);
    }
}
