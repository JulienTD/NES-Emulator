use crate::cpu6502::{CPU, StatusFlag};
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handle_plp(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let popped_status = self.pop_u8();

        // The B and U flags are not affected by PLP.
        // We need to preserve their current state from the status register.
        let b_flag_mask = 1 << (StatusFlag::BreakCommand as u8);
        let u_flag_mask = 1 << (StatusFlag::Unused as u8);
        let preserved_mask = b_flag_mask | u_flag_mask;

        // Set the status register to the popped value, but ignore the B and U flags from the stack.
        // Then, merge in the preserved B and U flags from the current status register.
        self.status_register = (popped_status & !preserved_mask) | (self.status_register & preserved_mask);

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

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
