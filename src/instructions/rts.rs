use crate::cpu6502::CPU;

impl CPU {
    pub(crate) fn handleRTS(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        // RTS pulls the return address (minus one) from the stack, increments it,
        // and then sets the program counter to that address.
        let return_address_minus_one = self.pop_u16();
        self.program_counter = return_address_minus_one.wrapping_add(1);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_rts_returns_from_subroutine() {
        let mut cpu = new_cpu();
        // Simulate a JSR call by pushing a return address (minus one) to the stack.
        // If JSR was at 0x8000, it would push 0x8002. The return address is 0x8003.
        cpu.push_u16(0x8002);
        assert_eq!(cpu.stack_pointer, 0xFD);

        cpu.handleRTS(None, None);

        assert_eq!(cpu.program_counter, 0x8003, "PC should be set to the return address + 1");
        assert_eq!(cpu.stack_pointer, 0xFF, "Stack pointer should be restored");
    }
}
