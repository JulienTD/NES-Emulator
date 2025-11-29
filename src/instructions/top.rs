use crate::cpu6502::CPU;

impl CPU {
    pub(crate) fn handle_top(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
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
    fn test_top_does_nothing() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // Set some initial state to ensure it doesn't change
        cpu.accumulator = 0xAA;
        cpu.x_register = 0xBB;
        cpu.status_register = 0b11001100;

        let cycles = cpu.handle_top(None, None);

        assert_eq!(cycles, 0, "top should not return extra cycles");
        assert_eq!(cpu.accumulator, 0xAA, "Accumulator should not change");
        assert_eq!(cpu.x_register, 0xBB, "X register should not change");
        assert_eq!(cpu.status_register, 0b11001100, "Status register should not change");
    }
}
