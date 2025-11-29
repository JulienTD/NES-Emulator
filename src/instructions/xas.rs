use crate::cpu6502::CPU;

impl CPU {
    // XAS (SHS/TAS) — AND X with A, store result to stack pointer S, then store S & (HIGH(arg)+1) into memory.
    // S = X & A
    // M = S & (HIGH(arg) + 1)
    // No flags affected.
    pub(crate) fn handle_xas(& mut self, _opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let address = opt_address.expect("BUG: address of XAS should be present");

        let s = self.x_register & self.accumulator;
        // store into stack pointer
        self.stack_pointer = s;

        let high = (address >> 8) as u8;
        let mem_val = s & high.wrapping_add(1);

        self.write_u8(address, mem_val);
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
    fn test_xas_stores_to_sp_and_memory() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));

        cpu.x_register = 0xFF;
        cpu.accumulator = 0x0F; // S = 0x0F

        // Pick writable address whose high byte is 0x03 -> high+1 = 0x04
        let addr: u16 = 0x0302;
        cpu.write_u8(addr, 0x00);

        let _ = cpu.handle_xas(None, Some(addr));

        // SP updated
        assert_eq!(cpu.stack_pointer, 0x0F);
        // Memory written: S & (0x03+1) = 0x0F & 0x04 = 0x04
        assert_eq!(cpu.read_u8(addr), 0x04);
    }

    #[test]
    fn test_xas_high_plus_one_zeroes_memory() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.x_register = 0xAA;
        cpu.accumulator = 0x55; // S = 0x00

        // high byte 0x01 => high+1 = 0x02
        let addr: u16 = 0x0110;
        cpu.write_u8(addr, 0xFF);

        let _ = cpu.handle_xas(None, Some(addr));

        assert_eq!(cpu.stack_pointer, 0x00);
        // S & 0x02 = 0x00
        assert_eq!(cpu.read_u8(addr), 0x00);
    }
}
