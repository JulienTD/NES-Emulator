use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handle_php(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
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
    use crate::bus::Bus;
    use crate::cpu6502::{new_cpu, StatusFlag};
    use crate::rom::Rom;

    #[test]
    fn test_php_pushes_status_to_stack() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.set_status_flag(StatusFlag::Carry, true); // Set C to 1
        cpu.set_status_flag(StatusFlag::Negative, true); // Set N to 1
        cpu.set_status_flag(StatusFlag::InterruptDisable, false); // Ensure I is cleared so initial status is 0b1000_0001

        cpu.handle_php(None, None);

        let pushed_status = cpu.read_u8(0x01FF);
        // Expected status on stack: 0b1011_0001 (B and U flags are set)
        assert_eq!(pushed_status, 0b1011_0001);
        assert_eq!(cpu.stack_pointer, 0xFE, "Stack pointer should decrement");
    }

    #[test]
    fn test_php_always_sets_b_and_u_in_pushed_value() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // Clear all flags
        cpu.status_register = 0x00;
        let initial_sp = cpu.stack_pointer;
        cpu.handle_php(None, None);
        let pushed = cpu.read_u8(0x0100 + initial_sp as u16);
        // B (bit 4) and U (bit 5) must be set
        assert_ne!(pushed & (1 << 4), 0, "B flag must be set in pushed status");
        assert_ne!(pushed & (1 << 5), 0, "U flag must be set in pushed status");
    }

    #[test]
    fn test_php_does_not_modify_status_register_itself() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let initial_status = cpu.status_register;
        cpu.handle_php(None, None);
        assert_eq!(cpu.status_register, initial_status, "PHP should not alter the actual status register");
    }

    #[test]
    fn test_php_stack_pointer_decremented_by_one() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let initial_sp = cpu.stack_pointer;
        cpu.handle_php(None, None);
        assert_eq!(cpu.stack_pointer, initial_sp.wrapping_sub(1));
    }

    #[test]
    fn test_php_returns_zero_extra_cycles() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let extra = cpu.handle_php(None, None);
        assert_eq!(extra, 0);
    }
}
