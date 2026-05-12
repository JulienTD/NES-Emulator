use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_arr(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of ARR should be present");

        // AND with accumulator
        let temp = self.accumulator & value;

        // Perform ROR on temp using current carry
        let old_carry = if self.get_status_flag(StatusFlag::Carry) { 1 } else { 0 };
        // New carry is old bit0
        self.set_status_flag(StatusFlag::Carry, (temp & 0x01) != 0);
        let result = (temp >> 1) | (old_carry << 7);

        // Set accumulator
        self.accumulator = result;

        // Set Zero and Negative
        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, (result & 0x80) != 0);

        // Determine V and C from bits 5 and 6
        let bit5 = (result & 0x20) != 0;
        let bit6 = (result & 0x40) != 0;
        if bit5 && bit6 {
            self.set_status_flag(StatusFlag::Carry, true);
            self.set_status_flag(StatusFlag::Overflow, false);
        } else if !bit5 && !bit6 {
            self.set_status_flag(StatusFlag::Carry, false);
            self.set_status_flag(StatusFlag::Overflow, false);
        } else if bit5 && !bit6 {
            self.set_status_flag(StatusFlag::Carry, false);
            self.set_status_flag(StatusFlag::Overflow, true);
        } else { // !bit5 && bit6
            self.set_status_flag(StatusFlag::Carry, true);
            self.set_status_flag(StatusFlag::Overflow, true);
        }

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu6502::{new_cpu, StatusFlag};
    use crate::bus::Bus;
    use crate::rom::Rom;

    #[test]
    fn test_arr_basic() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.set_status_flag(StatusFlag::Carry, true);
        cpu.accumulator = 0b0000_0011; // & operand will keep it similar
        let _ = cpu.handle_arr(Some(0b0000_0011), None);
        // After AND temp = 3, old carry 1 means result = (3 >> 1) | 0x80 = 0x81
        assert_eq!(cpu.accumulator, 0x81);
        assert!(cpu.get_status_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_arr_zero_flag_when_result_is_zero() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // AND of 0x00 with anything is 0; carry is 0 so result stays 0
        cpu.set_status_flag(StatusFlag::Carry, false);
        cpu.accumulator = 0x00;
        let _ = cpu.handle_arr(Some(0xFF), None);
        // temp = 0, result = 0 >> 1 | 0 = 0
        assert_eq!(cpu.accumulator, 0x00);
        assert!(cpu.get_status_flag(StatusFlag::Zero));
    }

    #[test]
    fn test_arr_carry_set_when_bits5_and_6_both_set() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // We need result to have bits 5 and 6 set, i.e. result = 0b0110_0000 = 0x60
        // result = (temp >> 1) | (old_carry << 7)
        // With old_carry = 0: result = temp >> 1 = 0x60 => temp = 0xC0
        cpu.set_status_flag(StatusFlag::Carry, false);
        cpu.accumulator = 0xFF;
        // AND with 0xC0 -> temp = 0xC0; shift -> 0x60; bits 5 and 6 both set
        let _ = cpu.handle_arr(Some(0xC0), None);
        assert_eq!(cpu.accumulator, 0x60);
        assert!(cpu.get_status_flag(StatusFlag::Carry));
        assert!(!cpu.get_status_flag(StatusFlag::Overflow));
    }

    #[test]
    fn test_arr_carry_clear_overflow_set_when_bit5_set_bit6_clear() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // result needs bit5=1, bit6=0 => result = 0b0010_0000 = 0x20
        // With old_carry=0: temp >> 1 = 0x20 => temp = 0x40
        cpu.set_status_flag(StatusFlag::Carry, false);
        cpu.accumulator = 0xFF;
        let _ = cpu.handle_arr(Some(0x40), None);
        assert_eq!(cpu.accumulator, 0x20);
        assert!(!cpu.get_status_flag(StatusFlag::Carry));
        assert!(cpu.get_status_flag(StatusFlag::Overflow));
    }
}
