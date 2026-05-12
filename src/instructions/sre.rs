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

        let value = cpu.read_u8(addr);
        let _ = cpu.handle_sre(Some(value), Some(addr));
        // shifted = 0b0000_0001
        assert_eq!(cpu.read_u8(addr), 0b0000_0001);
        // accumulator ^= shifted => 0b0101_0100
        assert_eq!(cpu.accumulator, 0b0101_0100);
        assert!(cpu.get_status_flag(StatusFlag::Carry));
    }

    #[test]
    fn test_sre_zero_flag_when_accumulator_becomes_zero() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0210;
        // shifted = 0x01 >> 1 = 0x00; A ^= 0x00 => A unchanged but we want A=0 after XOR
        // Use A = 0x01 and memory = 0x02 => shifted = 0x01, A ^= 0x01 = 0x00
        cpu.write_u8(addr, 0x02);
        cpu.accumulator = 0x01;
        let value = cpu.read_u8(addr);
        let _ = cpu.handle_sre(Some(value), Some(addr));
        assert_eq!(cpu.accumulator, 0x00);
        assert!(cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Carry)); // bit0 of 0x02 is 0
    }

    #[test]
    fn test_sre_negative_flag_when_accumulator_bit7_set() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0220;
        // shifted = 0x02 >> 1 = 0x01; A = 0x80 ^ 0x01 = 0x81 (bit7 set)
        cpu.write_u8(addr, 0x02);
        cpu.accumulator = 0x80;
        let value = cpu.read_u8(addr);
        let _ = cpu.handle_sre(Some(value), Some(addr));
        assert_eq!(cpu.accumulator, 0x81);
        assert!(cpu.get_status_flag(StatusFlag::Negative));
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
    }

    #[test]
    fn test_sre_carry_cleared_when_bit0_of_memory_is_zero() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0230;
        // memory = 0x04, bit0 = 0 => carry cleared
        cpu.write_u8(addr, 0x04);
        cpu.accumulator = 0x00;
        let value = cpu.read_u8(addr);
        let _ = cpu.handle_sre(Some(value), Some(addr));
        // shifted = 0x02, A = 0x00 ^ 0x02 = 0x02
        assert_eq!(cpu.accumulator, 0x02);
        assert!(!cpu.get_status_flag(StatusFlag::Carry));
    }
}
