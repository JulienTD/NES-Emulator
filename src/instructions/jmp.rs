use crate::cpu6502::CPU;

impl CPU {
    pub(crate) fn handle_jmp(& mut self, _opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let address = opt_address.expect("BUG: address of JMP should be present");
        self.program_counter = address;
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu6502::new_cpu;
    use crate::rom::Rom;

    #[test]
    fn test_jmp_sets_program_counter() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.handle_jmp( None, Some(0x1234));
        assert_eq!(cpu.program_counter, 0x1234);
    }

}
