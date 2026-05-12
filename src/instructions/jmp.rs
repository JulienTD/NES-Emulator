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

    #[test]
    fn test_jmp_to_address_zero() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.program_counter = 0x8000;
        cpu.handle_jmp(None, Some(0x0000));
        assert_eq!(cpu.program_counter, 0x0000);
    }

    #[test]
    fn test_jmp_to_address_ffff() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.handle_jmp(None, Some(0xFFFF));
        assert_eq!(cpu.program_counter, 0xFFFF);
    }

    #[test]
    fn test_jmp_returns_zero_extra_cycles() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let extra = cpu.handle_jmp(None, Some(0x0200));
        assert_eq!(extra, 0);
    }

    #[test]
    fn test_jmp_does_not_affect_status_register() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let initial_status = cpu.status_register;
        cpu.handle_jmp(None, Some(0x0300));
        assert_eq!(cpu.status_register, initial_status);
    }

}
