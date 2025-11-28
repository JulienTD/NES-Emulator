use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_rol(& mut self, opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of ROL should be present");

        // Get the current carry flag value to be rotated into bit 0
        let old_carry = if self.get_status_flag(StatusFlag::Carry) { 1 } else { 0 };

        // The new carry will be the old bit 7
        self.set_status_flag(StatusFlag::Carry, (value & 0x80) != 0);

        // Perform the rotation: shift left and bring old carry into bit 0
        let result = (value << 1) | old_carry;

        // Set Zero and Negative flags based on the result
        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, (result & 0x80) != 0);

        // Store the result back
        if let Some(address) = opt_address {
            self.write_u8(address, result);
        } else {
            // Accumulator mode
            self.accumulator = result;
        }

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu6502::{new_cpu, StatusFlag};
    use crate::rom::Rom;

    #[test]
    fn test_rol_accumulator_with_carry() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.set_status_flag(StatusFlag::Carry, true); // Set initial carry
        cpu.accumulator = 0b1010_1010;
        cpu.handle_rol(Some(cpu.accumulator), None);

        assert_eq!(cpu.accumulator, 0b0101_0101, "Result should be rotated with carry as new bit 0");
        assert!(cpu.get_status_flag(StatusFlag::Carry), "New carry should be set from old bit 7");
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_rol_memory_no_carry() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let address = 0x0200;
        cpu.write_u8(address, 0b0101_0101);
        cpu.set_status_flag(StatusFlag::Carry, false); // Clear initial carry
        cpu.handle_rol(Some(0b0101_0101), Some(address));

        assert_eq!(cpu.read_u8(address), 0b1010_1010, "Result should be rotated with 0 as new bit 0");
        assert!(!cpu.get_status_flag(StatusFlag::Carry), "New carry should be clear from old bit 7");
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert!(cpu.get_status_flag(StatusFlag::Negative), "Negative flag should be set");
    }
}
