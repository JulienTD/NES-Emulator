use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_aac(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of ANC should be present");

        // ANC is an unofficial opcode: AND the accumulator with the operand
        // then set the Carry flag to the result's bit 7. Also update Z and N.
        let result = self.accumulator & value;
        self.accumulator = result;

        // Update flags: Zero, Negative
        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, (result & 0x80) != 0);

        // ANC sets the Carry flag to the high bit of the result
        self.set_status_flag(StatusFlag::Carry, (result & 0x80) != 0);

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
    fn test_anc_sets_accumulator_and_flags() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));

        // Case: result has high bit clear => carry false, negative false
        cpu.accumulator = 0b0110_1111;
        let extra = cpu.handle_aac(Some(0b1111_1111), None);
        assert_eq!(extra, 0);
        assert_eq!(cpu.accumulator, 0b0110_1111);
        assert!(!cpu.get_status_flag(StatusFlag::Carry));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
        assert!(!cpu.get_status_flag(StatusFlag::Zero));

        // Case: result high bit set => carry true, negative true
        cpu.accumulator = 0b1000_0000;
        let _ = cpu.handle_aac(Some(0b1111_1111), None);
        assert_eq!(cpu.accumulator, 0b1000_0000);
        assert!(cpu.get_status_flag(StatusFlag::Carry));
        assert!(cpu.get_status_flag(StatusFlag::Negative));

        // Case: result zero => zero flag set, carry false
        cpu.accumulator = 0b0000_0000;
        let _ = cpu.handle_aac(Some(0b0000_0000), None);
        assert_eq!(cpu.accumulator, 0b0000_0000);
        assert!(cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Carry));
    }
}
