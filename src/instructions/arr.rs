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
}
