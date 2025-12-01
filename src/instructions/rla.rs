use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    // RLA — rotate memory left (like ROL) then AND accumulator with memory
    // Flags: N,Z,C (based on AND result and rotation carry)
    pub(crate) fn handle_rla(& mut self, opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of RLA should be present");

        // ROL on memory value using current carry
        let old_carry = if self.get_status_flag(StatusFlag::Carry) { 1 } else { 0 };
        let new_carry = (value & 0x80) != 0;
        let rotated = (value << 1) | old_carry;

        if let Some(address) = opt_address {
            self.write_u8(address, rotated);
        }

        // AND accumulator with rotated value
        self.accumulator &= rotated;

        // Update flags
        self.set_status_flag(StatusFlag::Carry, new_carry);
        self.set_status_flag(StatusFlag::Zero, self.accumulator == 0);
        self.set_status_flag(StatusFlag::Negative, (self.accumulator & 0x80) != 0);

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu6502::new_cpu;
    use crate::bus::Bus;
    use crate::rom::Rom;
    use crate::cpu6502::StatusFlag;

    #[test]
    fn test_rla_memory_and_accumulator() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0200;
        cpu.write_u8(addr, 0b0100_0000);
        cpu.accumulator = 0b1111_1111;
        cpu.set_status_flag(StatusFlag::Carry, 1 == 1); // set carry -> 1

        let _ = cpu.handle_rla(Some(cpu.read_u8(addr)), Some(addr));

        // rotated = (0b0100_0000 << 1) | 1 = 0b1000_0001
        assert_eq!(cpu.read_u8(addr), 0b1000_0001);
        // accumulator = 0b1111_1111 & 0b1000_0001 = 0b1000_0001
        assert_eq!(cpu.accumulator, 0b1000_0001);
        // original memory had bit7 = 0 so ROL cleared carry
        assert!(!cpu.get_status_flag(StatusFlag::Carry));
        assert!(cpu.get_status_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_rla_sets_carry_when_high_bit() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0210;
        cpu.write_u8(addr, 0b1000_0000); // high bit set
        cpu.accumulator = 0b1111_1111;
        cpu.set_status_flag(StatusFlag::Carry, false);

        let _ = cpu.handle_rla(Some(cpu.read_u8(addr)), Some(addr));

        // rotated = (0b1000_0000 << 1) | 0 = 0b0000_0000
        assert_eq!(cpu.read_u8(addr), 0b0000_0000);
        // accumulator = 0xFF & 0x00 = 0x00
        assert_eq!(cpu.accumulator, 0x00);
        assert!(cpu.get_status_flag(StatusFlag::Zero));
        // carry should be set from original high bit
        assert!(cpu.get_status_flag(StatusFlag::Carry));
    }
}
