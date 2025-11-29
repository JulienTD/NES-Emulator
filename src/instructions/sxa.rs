use crate::cpu6502::{CPU};

impl CPU {
    // SXA (SHX) - AND X register with the high byte of the argument + 1, store result into memory
    // M = X & (HIGH(arg) + 1)
    // No flags affected.
    pub(crate) fn handle_sxa(& mut self, _opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let address = opt_address.expect("BUG: address of SXA should be present");

        let high = (address >> 8) as u8;
        let result = self.x_register & high.wrapping_add(1);
        self.write_u8(address, result);

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;
    use crate::bus::Bus;
    use crate::rom::Rom;

    #[test]
    fn test_sxa_stores_x_and_high_plus_one() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // Put some arbitrary X
        cpu.x_register = 0xFF;

        // We'll use address 0x0302, high byte = 0x03
        let addr: u16 = 0x0302;
        // ensure memory at addr is different
        cpu.write_u8(addr, 0x00);

        let _ = cpu.handle_sxa(None, Some(addr));

        // high = 0x03 ; high+1 = 0x04 ; result = 0xFF & 0x04 = 0x04
        assert_eq!(cpu.read_u8(addr), 0x04);
    }

    #[test]
    fn test_sxa_high_plus_one_behavior() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.x_register = 0xAA;
        // Choose a writable address whose high byte is 0x01 -> high+1 = 0x02
        let addr: u16 = 0x0110;
        cpu.write_u8(addr, 0xFF);

        let _ = cpu.handle_sxa(None, Some(addr));

        // high = 0x01 ; high+1 = 0x02 ; result = X & 0x02 = 0x02
        assert_eq!(cpu.read_u8(addr), 0x02);
    }
}
