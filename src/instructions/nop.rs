use crate::cpu6502::CPU;

impl CPU {
    pub(crate) fn handle_nop(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        // NOP does nothing.
        return 0;
    }
}

#[cfg(test)]
mod tests {

    use crate::bus::Bus;
    use crate::cpu6502::new_cpu;
    use crate::rom::Rom;

    #[test]
    fn test_nop_does_nothing() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // Set some initial state to ensure it doesn't change
        cpu.accumulator = 0xAA;
        cpu.x_register = 0xBB;
        cpu.status_register = 0b11001100;

        let cycles = cpu.handle_nop(None, None);

        assert_eq!(cycles, 0, "NOP should not return extra cycles");
        assert_eq!(cpu.accumulator, 0xAA, "Accumulator should not change");
        assert_eq!(cpu.x_register, 0xBB, "X register should not change");
        assert_eq!(cpu.status_register, 0b11001100, "Status register should not change");
    }

    #[test]
    fn test_nop_does_not_change_program_counter() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.program_counter = 0x1234;
        cpu.handle_nop(None, None);
        assert_eq!(cpu.program_counter, 0x1234, "NOP should not modify the program counter directly");
    }

    #[test]
    fn test_nop_does_not_change_stack_pointer() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let initial_sp = cpu.stack_pointer;
        cpu.handle_nop(None, None);
        assert_eq!(cpu.stack_pointer, initial_sp, "NOP should not modify the stack pointer");
    }

    #[test]
    fn test_nop_does_not_change_y_register() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.y_register = 0xCC;
        cpu.handle_nop(None, None);
        assert_eq!(cpu.y_register, 0xCC, "NOP should not modify the Y register");
    }
}
