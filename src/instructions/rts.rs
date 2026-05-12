use crate::cpu6502::CPU;

impl CPU {
    pub(crate) fn handle_rts(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        // RTS pulls the return address (minus one) from the stack, increments it,
        // and then sets the program counter to that address.
        let return_address_minus_one = self.pop_u16();
        self.program_counter = return_address_minus_one.wrapping_add(1);
        return 0;
    }
}

#[cfg(test)]
mod tests {

    use crate::bus::Bus;
    use crate::cpu6502::new_cpu;
    use crate::rom::Rom;

    #[test]
    fn test_rts_returns_from_subroutine() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // Simulate a JSR call by pushing a return address (minus one) to the stack.
        // If JSR was at 0x8000, it would push 0x8002. The return address is 0x8003.
        cpu.push_u16(0x8002);
        assert_eq!(cpu.stack_pointer, 0xFD);

        cpu.handle_rts(None, None);

        assert_eq!(cpu.program_counter, 0x8003, "PC should be set to the return address + 1");
        assert_eq!(cpu.stack_pointer, 0xFF, "Stack pointer should be restored");
    }

    #[test]
    fn test_rts_stack_pointer_incremented_by_two() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.push_u16(0x0100);
        let sp_after_push = cpu.stack_pointer;
        cpu.handle_rts(None, None);
        assert_eq!(cpu.stack_pointer, sp_after_push.wrapping_add(2));
    }

    #[test]
    fn test_rts_does_not_affect_status_register() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.push_u16(0x0200);
        let initial_status = cpu.status_register;
        cpu.handle_rts(None, None);
        assert_eq!(cpu.status_register, initial_status);
    }

    #[test]
    fn test_rts_returns_zero_extra_cycles() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.push_u16(0x0300);
        let extra = cpu.handle_rts(None, None);
        assert_eq!(extra, 0);
    }

    #[test]
    fn test_jsr_followed_by_rts_restores_pc() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.program_counter = 0x8000;
        // JSR pushes PC+2 = 0x8002, then jumps to 0x9000
        cpu.handle_jsr(None, Some(0x9000));
        assert_eq!(cpu.program_counter, 0x9000);
        // RTS pops 0x8002 and adds 1 => PC = 0x8003
        cpu.handle_rts(None, None);
        assert_eq!(cpu.program_counter, 0x8003);
    }
}
