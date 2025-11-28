use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_rti(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let popped_status = self.pop_u8();
        self.program_counter = self.pop_u16();

        // The B and U flags are not affected by RTI.
        // We need to preserve their current state from the status register.
        let b_flag_mask = 1 << (StatusFlag::BreakCommand as u8);
        let u_flag_mask = 1 << (StatusFlag::Unused as u8);
        let preserved_mask = b_flag_mask | u_flag_mask;

        self.status_register = (popped_status & !preserved_mask) | (self.status_register & preserved_mask);

        return 0;
    }
}

#[cfg(test)]
mod tests {

    use crate::bus::Bus;
    use crate::cpu6502::new_cpu;
    use crate::rom::Rom;

    #[test]
    fn test_rti_restores_status_and_pc() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let return_address = 0x1234;
        let status_on_stack = 0b1011_0101; // A status with B and U flags set

        // Simulate an interrupt by pushing PC and status
        cpu.push_u16(return_address);
        cpu.push_u8(status_on_stack);

        cpu.handle_rti(None, None);

        assert_eq!(cpu.program_counter, return_address, "Program counter should be restored");

        // The status register should be 0b1010_0101 (165).
        // N (Bit 7) = 1 (From Stack)
        // V (Bit 6) = 0 (From Stack)
        // U (Bit 5) = 1 (FORCED HIGH by hardware nature)
        // B (Bit 4) = 0 (FORCED LOW, B flag never exists in register)
        // D (Bit 3) = 0 (From Stack)
        // I (Bit 2) = 1 (From Stack)
        // Z (Bit 1) = 0 (From Stack)
        // C (Bit 0) = 1 (From Stack)
        assert_eq!(cpu.status_register, 0b1010_0101, "Status register should be restored, B ignored, U set high");
        assert_eq!(cpu.stack_pointer, 0xFF, "Stack pointer should be restored to its original state");
    }
}
