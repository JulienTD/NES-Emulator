use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_plp(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let popped_status = self.pop_u8();

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
    fn test_plp_pulls_status_from_stack() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // Push a status with C=1, N=1, B=1, U=1 (0b10110001)
        cpu.push_u8(0b10110001);

        cpu.handle_plp(None, None);

        // The status register should be:
        // N=1 (From Stack)
        // V=0 (From Stack)
        // U=1 (Ignored from Stack, but kept as 1 from CPU state)
        // B=0 (Ignored from Stack, B flag is 0 in register)
        // D=0 (From Stack)
        // I=0 (From Stack)
        // Z=0 (From Stack)
        // C=1 (From Stack)
        // Result: 10100001
        assert_eq!(cpu.status_register, 0b10100001);
        assert_eq!(cpu.stack_pointer, 0xFF, "Stack pointer should increment");
    }

    #[test]
    fn test_plp_clears_b_flag_regardless_of_stack() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // Push a status with B (bit 4) set
        cpu.push_u8(0b0001_0000);
        cpu.handle_plp(None, None);
        // B flag should be cleared (bit 4 = 0)
        assert_eq!(cpu.status_register & (1 << 4), 0, "B flag must be cleared after PLP");
    }

    #[test]
    fn test_plp_sets_u_flag_regardless_of_stack() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // Push a status with U (bit 5) cleared
        cpu.push_u8(0b0000_0000);
        cpu.handle_plp(None, None);
        // U flag should be forced set (bit 5 = 1)
        assert_ne!(cpu.status_register & (1 << 5), 0, "U flag must be set after PLP");
    }

    #[test]
    fn test_plp_stack_pointer_incremented_by_one() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.push_u8(0xFF);
        let sp_after_push = cpu.stack_pointer;
        cpu.handle_plp(None, None);
        assert_eq!(cpu.stack_pointer, sp_after_push.wrapping_add(1));
    }

    #[test]
    fn test_plp_returns_zero_extra_cycles() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.push_u8(0x00);
        let extra = cpu.handle_plp(None, None);
        assert_eq!(extra, 0);
    }

    #[test]
    fn test_plp_preserves_carry_and_negative_from_stack() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // Push: N=1, C=1, B=0, U=0 => 0b1000_0001
        cpu.push_u8(0b1000_0001);
        cpu.handle_plp(None, None);
        // Carry (bit 0) should be set
        assert_ne!(cpu.status_register & 0x01, 0, "Carry should be set");
        // Negative (bit 7) should be set
        assert_ne!(cpu.status_register & 0x80, 0, "Negative should be set");
    }
}
