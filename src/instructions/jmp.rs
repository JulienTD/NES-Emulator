use crate::cpu6502::CPU;
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handleJMP(& mut self, _opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let address = opt_address.expect("BUG: address of JMP should be present");
        self.program_counter = address;
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::{AddressingMode, new_cpu};

    #[test]
    fn test_jmp_sets_program_counter() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.handleJMP( None, Some(0x1234));
        assert_eq!(cpu.program_counter, 0x1234);
    }

}
