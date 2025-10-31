use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handlePHP(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        // When PHP is used, the status register is pushed to the stack
        // with the Break (B) and Unused (U) flags set to 1.
        let mut status = self.status_register;
        status |= 1 << (StatusFlag::BreakCommand as u8);
        status |= 1 << (StatusFlag::Unused as u8);
        self.push_u8(status);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_php_pushes_status_to_stack() {
        let mut cpu = new_cpu();
        cpu.set_status_flag(StatusFlag::Carry, true); // Set C to 1
        cpu.set_status_flag(StatusFlag::Negative, true); // Set N to 1
        // Initial status is 0b1000_0001

        cpu.handlePHP(None, None);

        let pushed_status = cpu.read_u8(0x01FF);
        // Expected status on stack: 0b1011_0001 (B and U flags are set)
        assert_eq!(pushed_status, 0b1011_0001);
        assert_eq!(cpu.stack_pointer, 0xFE, "Stack pointer should decrement");
    }
}
