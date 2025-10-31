use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handlePLP(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
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
        let mut cpu = new_cpu();
        // Push a status with C=1, N=1, B=1, U=1 (0b10110001)
        cpu.push_u8(0b10110001);

        cpu.handlePLP(None, None);

        // The status register should be 0b10000001. C and N are set from the stack,
        // but B and U are ignored and retain their original (0) value.
        assert_eq!(cpu.status_register, 0b10000001);
        assert_eq!(cpu.stack_pointer, 0xFF, "Stack pointer should increment");
    }
}
