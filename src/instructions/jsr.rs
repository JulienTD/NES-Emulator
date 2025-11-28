use crate::cpu6502::CPU;

impl CPU {
    pub(crate) fn handle_jsr(& mut self, _opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let target_address = opt_address.expect("BUG: address of JSR should be present");

        // JSR is a 3-byte instruction. It pushes the address of its last byte (PC+2)
        // onto the stack. This serves as the "return address minus one" for RTS.
        let return_address = self.program_counter + 2;
        self.push_u16(return_address);

        // Set the program counter to the target address to jump to the subroutine.
        self.program_counter = target_address;
        return 0;
    }
}

#[cfg(test)]
mod tests {

    use crate::bus::Bus;
    use crate::cpu6502::new_cpu;
    use crate::rom::Rom;

    #[test]
    fn test_jsr_pushes_return_address_and_jumps() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.program_counter = 0x8000; // JSR is at 0x8000
        cpu.handle_jsr(None, Some(0x1234));

        assert_eq!(cpu.program_counter, 0x1234, "PC should jump to target address");
        assert_eq!(cpu.stack_pointer, 0xFD, "Stack pointer should be decremented by 2");
        // The address of the last byte of the instruction (0x8000 + 2 = 0x8002) should be pushed.
        assert_eq!(cpu.read_u16(0x01FE), 0x8002, "Return address (minus one) should be pushed to the stack");
    }
}
