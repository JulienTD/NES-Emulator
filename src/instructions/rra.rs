use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    // RRA — rotate right memory (like ROR) then ADC with accumulator
    // Flags: N,V,Z,C (ADC result)
    pub(crate) fn handle_rra(& mut self, opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of RRA should be present");

        // ROR on memory value using current carry
        let old_carry = if self.get_status_flag(StatusFlag::Carry) { 1 } else { 0 };
        let new_carry = (value & 0x01) != 0;
        let rotated = (value >> 1) | (old_carry << 7);

        if let Some(address) = opt_address {
            self.write_u8(address, rotated);
        }

        // ROR updated carry should be used as carry-in for ADC
        self.set_status_flag(StatusFlag::Carry, new_carry);
        let carry_in = if self.get_status_flag(StatusFlag::Carry) { 1 } else { 0 };
        let sum = (self.accumulator as u16) + (rotated as u16) + carry_in;
        let result = sum as u8;

        // Flags
        self.set_status_flag(StatusFlag::Carry, sum > 0xFF);
        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, (result & 0x80) != 0);

        // Overflow detection (signed)
        let signed_a = self.accumulator as i8;
        let signed_b = rotated as i8;
        let signed_r = result as i8;
        let overflow = (signed_a >= 0 && signed_b >= 0 && signed_r < 0) || (signed_a < 0 && signed_b < 0 && signed_r >= 0);
        self.set_status_flag(StatusFlag::Overflow, overflow);

        self.accumulator = result;
        // new carry from rotation also influences final carry already set by ADC; leave ADC carry

        // final carry comes from ADC result (already set above)

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu6502::{new_cpu, StatusFlag};
    use crate::rom::Rom;

    #[test]
    fn test_rra_memory_adds_to_accumulator_and_rotates() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0200;
        cpu.write_u8(addr, 0b0000_0011);
        cpu.accumulator = 0x01;
        cpu.set_status_flag(StatusFlag::Carry, true);

        let _ = cpu.handle_rra(Some(cpu.read_u8(addr)), Some(addr));

        // rotated = (3 >> 1) | (1 << 7) = 0b1000_0001 = 0x81
        assert_eq!(cpu.read_u8(addr), 0x81);
        // accumulator = oldA + rotated + carry_in = 0x01 + 0x81 + 1 = 0x83
        assert_eq!(cpu.accumulator, 0x83);
    }

    #[test]
    fn test_rra_uses_rotation_carry_as_adc_carry_in() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0300;
        // memory LSB = 1 -> rotation sets carry to 1
        cpu.write_u8(addr, 0b0000_0001);
        cpu.accumulator = 0x00;
        cpu.set_status_flag(StatusFlag::Carry, false); // old carry is 0

        let _ = cpu.handle_rra(Some(cpu.read_u8(addr)), Some(addr));

        // rotated = (1 >> 1) | (0<<7) = 0
        assert_eq!(cpu.read_u8(addr), 0x00);
        // carry_in should be 1 (rotation carry), so A = 0 + 0 + 1 = 1
        assert_eq!(cpu.accumulator, 0x01);
        // final carry is from ADC (no overflow here)
        assert!(!cpu.get_status_flag(StatusFlag::Carry));
    }

    #[test]
    fn test_rra_adc_overflow_and_carry() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0310;
        // memory LSB = 0 -> rotation sets carry 0, but rotated has high bit set from old carry
        cpu.write_u8(addr, 0b0000_0000);
        cpu.accumulator = 0xFF;
        cpu.set_status_flag(StatusFlag::Carry, true); // old carry 1 -> rotated MSB will be 1

        // rotated = (0 >> 1) | (1 << 7) = 0x80
        // sum = 0xFF + 0x80 + carry_in(=rotation carry=0) -> if carry_in used would be 0 but here rotation carry = 0
        let _ = cpu.handle_rra(Some(cpu.read_u8(addr)), Some(addr));

        // rotated written to memory
        assert_eq!(cpu.read_u8(addr), 0x80);
        // accumulator = 0xFF + 0x80 + 0 (carry from rotation) = 0x7F with carry out
        assert_eq!(cpu.accumulator, 0x7F);
        assert!(cpu.get_status_flag(StatusFlag::Carry));
        // check overflow: signed 127 (7F) from negative + positive should set overflow
        assert!(cpu.get_status_flag(StatusFlag::Overflow));
    }
}
