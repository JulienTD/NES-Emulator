use crate::cpu6502::CPU;
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handlePHA(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        self.push_u8(self.accumulator);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_pha_pushes_accumulator_to_stack() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x42;
        let initial_sp = cpu.stack_pointer;

        let cycles = cpu.handlePHA(None, None);

        assert_eq!(cycles, 0, "PHA should not return extra cycles");
        assert_eq!(cpu.stack_pointer, initial_sp.wrapping_sub(1), "Stack pointer should decrement");
        assert_eq!(cpu.read_u8(0x0100 + initial_sp as u16), 0x42, "Accumulator value should be on the stack");
    }
}
