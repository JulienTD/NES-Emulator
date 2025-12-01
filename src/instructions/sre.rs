use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    // SRE — LSR memory then EOR with accumulator
    // Flags: N,Z,C
    pub(crate) fn handle_sre(& mut self, opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of SRE should be present");

        // LSR on memory
        let new_carry = (value & 0x01) != 0;
        let shifted = value >> 1;

        if let Some(address) = opt_address {
            self.write_u8(address, shifted);
        }

        // EOR accumulator with shifted value
        self.accumulator ^= shifted;

        self.set_status_flag(StatusFlag::Carry, new_carry);
        self.set_status_flag(StatusFlag::Zero, self.accumulator == 0);
        self.set_status_flag(StatusFlag::Negative, (self.accumulator & 0x80) != 0);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu6502::new_cpu;
    use crate::cpu6502::StatusFlag;
    use crate::rom::Rom;

    #[test]
    fn test_sre_shifts_and_eors() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0200;
        cpu.write_u8(addr, 0b0000_0011);
        cpu.accumulator = 0b0101_0101;

        let _ = cpu.handle_sre(Some(cpu.read_u8(addr)), Some(addr));
        // shifted = 0b0000_0001
        assert_eq!(cpu.read_u8(addr), 0b0000_0001);
        // accumulator ^= shifted => 0b0101_0100
        assert_eq!(cpu.accumulator, 0b0101_0100);
        assert!(cpu.get_status_flag(StatusFlag::Carry));
    }
}
