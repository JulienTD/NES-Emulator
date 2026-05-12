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
}
