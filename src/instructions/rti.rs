use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_rti(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let popped_status = self.pop_u8();
        self.program_counter = self.pop_u16();

        // B (bit 4) is not a real hardware flag; force it clear.
        // U (bit 5) is hardwired high; force it set.
        let b_flag_mask = 1 << (StatusFlag::BreakCommand as u8);
        let u_flag_mask = 1 << (StatusFlag::Unused as u8);

        self.status_register = (popped_status & !b_flag_mask) | u_flag_mask;

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

    #[test]
    fn test_rti_clears_b_flag_in_status() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.push_u16(0x0300);
        // Push status with B flag (bit 4) set
        cpu.push_u8(0b0001_0000);
        cpu.handle_rti(None, None);
        assert_eq!(cpu.status_register & (1 << 4), 0, "B flag must be cleared after RTI");
    }

    #[test]
    fn test_rti_forces_u_flag_set_in_status() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.push_u16(0x0300);
        // Push status with U flag (bit 5) cleared
        cpu.push_u8(0b0000_0000);
        cpu.handle_rti(None, None);
        assert_ne!(cpu.status_register & (1 << 5), 0, "U flag must be set after RTI");
    }

    #[test]
    fn test_rti_restores_pc_correctly() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.push_u16(0xABCD);
        cpu.push_u8(0x00);
        cpu.handle_rti(None, None);
        assert_eq!(cpu.program_counter, 0xABCD);
    }

    #[test]
    fn test_rti_stack_pointer_restored_after_brk() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let initial_sp = cpu.stack_pointer;
        cpu.program_counter = 0x8000;
        cpu.handle_brk(None, None);
        // BRK decremented SP by 3; RTI should restore it
        cpu.handle_rti(None, None);
        assert_eq!(cpu.stack_pointer, initial_sp);
    }

    #[test]
    fn test_rti_returns_zero_extra_cycles() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.push_u16(0x0100);
        cpu.push_u8(0x00);
        let extra = cpu.handle_rti(None, None);
        assert_eq!(extra, 0);
    }
}
