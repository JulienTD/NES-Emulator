use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_dcp(& mut self, opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of DCP should be present");
        let address = opt_address.expect("BUG: address of DCP should be present");

        let new_value = value.wrapping_sub(1);
        self.write_u8(address, new_value);

        // CMP logic: A - M
        let result = self.accumulator.wrapping_sub(new_value);
        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, (result & 0x80) != 0);
        self.set_status_flag(StatusFlag::Carry, self.accumulator >= new_value);

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Bus;
    use crate::cpu6502::new_cpu;
    use crate::cpu6502::StatusFlag;
    use crate::rom::Rom;

    #[test]
    fn test_dcp_decrements_memory_and_sets_cmp_flags() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0200;
        cpu.write_u8(addr, 0x05);
        cpu.accumulator = 0x06;

        let _ = cpu.handle_dcp(Some(cpu.read_u8(addr)), Some(addr));

        assert_eq!(cpu.read_u8(addr), 0x04);
        // 0x06 - 0x04 = 0x02 -> not zero, carry set
        assert!(cpu.get_status_flag(StatusFlag::Carry));
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
    }

    #[test]
    fn test_dcp_sets_zero_flag_when_equal() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0300;
        cpu.write_u8(addr, 0x05);
        // after decrement memory -> 0x04
        cpu.accumulator = 0x04;

        let _ = cpu.handle_dcp(Some(cpu.read_u8(addr)), Some(addr));

        assert_eq!(cpu.read_u8(addr), 0x04);
        // A == M -> zero set, carry set
        assert!(cpu.get_status_flag(StatusFlag::Zero));
        assert!(cpu.get_status_flag(StatusFlag::Carry));
    }

    #[test]
    fn test_dcp_sets_negative_flag_on_high_bit_result() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0400;
        // memory 0x81 -> decrement to 0x80
        cpu.write_u8(addr, 0x81);
        cpu.accumulator = 0x00;

        let _ = cpu.handle_dcp(Some(cpu.read_u8(addr)), Some(addr));

        assert_eq!(cpu.read_u8(addr), 0x80);
        // 0 - 0x80 = 0x80 -> negative set, carry cleared
        assert!(cpu.get_status_flag(StatusFlag::Negative));
        assert!(!cpu.get_status_flag(StatusFlag::Carry));
    }
}
