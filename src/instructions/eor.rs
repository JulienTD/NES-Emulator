use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_eor(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of EOR should be present");
        let result = self.accumulator ^ value;
        self.accumulator = result;
        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, result & 0x80 != 0 );
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu6502::{new_cpu, StatusFlag};
    use crate::rom::Rom;

    #[test]
    fn test_eor_sets_flags_correctly() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));

        // Test result > 0
        cpu.accumulator = 0b10101010;
        let extra = cpu.handle_eor(Some(0b01010101), None);
        assert_eq!(extra, 0);
        assert_eq!(cpu.accumulator, 0b11111111);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);

        // Test result == 0
        cpu.accumulator = 0b11110000;
        let extra = cpu.handle_eor(Some(0b11110000), None);
        assert_eq!(extra, 0);
        assert_eq!(cpu.accumulator, 0b00000000);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), false);

        // Test result < 0
        cpu.accumulator = 0b00001111;
        let extra = cpu.handle_eor(Some(0b11110000), None);
        assert_eq!(extra, 0);
        assert_eq!(cpu.accumulator, 0b11111111);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
    }
}
