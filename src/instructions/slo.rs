use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    // SLO — ASL memory then OR with accumulator
    // Flags: N,Z,C (from ASL and OR result)
    pub(crate) fn handle_slo(& mut self, opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of SLO should be present");

        // ASL on memory
        let new_carry = (value & 0x80) != 0;
        let rotated = value << 1;

        if let Some(address) = opt_address {
            self.write_u8(address, rotated);
        }

        // OR accumulator with rotated
        self.accumulator |= rotated;

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
    fn test_slo_shifts_and_ors() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0200;
        cpu.write_u8(addr, 0b0100_0000);
        cpu.accumulator = 0b0000_0001;

        let _ = cpu.handle_slo(Some(cpu.read_u8(addr)), Some(addr));
        // rotated = 0b1000_0000
        assert_eq!(cpu.read_u8(addr), 0b1000_0000);
        // accumulator OR rotated = 0b1000_0001
        assert_eq!(cpu.accumulator, 0b1000_0001);
        // original memory had bit7 = 0 so carry should be cleared
        assert!(!cpu.get_status_flag(StatusFlag::Carry));
        // accumulator has high bit set after OR -> negative
        assert!(cpu.get_status_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_slo_sets_carry_and_zero_when_memory_high_bit() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0210;
        cpu.write_u8(addr, 0b1000_0000); // bit7 set
        cpu.accumulator = 0x00;

        let _ = cpu.handle_slo(Some(cpu.read_u8(addr)), Some(addr));

        // rotated = 0b0000_0000 (shifted left) then OR with accumulator leaves 0
        assert_eq!(cpu.read_u8(addr), 0b0000_0000);
        assert_eq!(cpu.accumulator, 0x00);
        assert!(cpu.get_status_flag(StatusFlag::Carry));
        assert!(cpu.get_status_flag(StatusFlag::Zero));
    }
}
