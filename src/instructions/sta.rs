use crate::cpu6502::CPU;

impl CPU {
    pub(crate) fn handle_sta(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let address = _opt_address.expect("BUG: address of STA should be present");
        self.write_u8(address, self.accumulator);
        return 0;
    }
}

#[cfg(test)]
mod tests {

    use crate::bus::Bus;
    use crate::cpu6502::new_cpu;
    use crate::rom::Rom;

    #[test]
    fn test_sta_stores_accumulator_in_memory() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let address = 0x0200;
        cpu.accumulator = 0x42;
        let initial_status = cpu.status_register;

        let cycles = cpu.handle_sta(None, Some(address));

        assert_eq!(cycles, 0, "STA should not return extra cycles");
        assert_eq!(cpu.read_u8(address), 0x42, "Accumulator value should be stored at the address");
        assert_eq!(cpu.status_register, initial_status, "STA should not affect any flags");
    }
}