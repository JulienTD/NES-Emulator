use crate::cpu6502::{CPU, StatusFlag};
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handleLSR(& mut self, opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of LSR should be present");

        // Set Carry flag (C) - set if bit 0 of original value was 1
        self.set_status_flag(StatusFlag::Carry, (value & 0x01) != 0);

        // Perform the shift
        let result = value >> 1;

        // Set Zero flag (Z) - set if result is 0
        self.set_status_flag(StatusFlag::Zero, result == 0);

        // Set Negative flag (N) - should always cleared as bit 7 is shifted to 0
        self.set_status_flag(StatusFlag::Negative, (result & 0x80) != 0);

        // If an address is present, it's a memory operation. Otherwise, it's accumulator.
        if let Some(address) = opt_address {
            self.write_u8(address, result);
        } else {
            self.accumulator = result;
        }

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_lsr_accumulator() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0b0000_0011; // Value is 3, bit 0 is 1
        cpu.handleLSR(Some(cpu.accumulator), None);
        assert_eq!(cpu.accumulator, 0b0000_0001); // Result is 1
        assert!(cpu.get_status_flag(StatusFlag::Carry), "Carry should be set from bit 0");
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_lsr_memory() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let address = 0x0200;
        cpu.write_u8(address, 0b1000_0010); // Value is 130, bit 0 is 0
        cpu.handleLSR(Some(0b1000_0010), Some(address));
        assert_eq!(cpu.read_u8(address), 0b0100_0001); // Result is 65
        assert!(!cpu.get_status_flag(StatusFlag::Carry), "Carry should be clear from bit 0");
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
    }
}
