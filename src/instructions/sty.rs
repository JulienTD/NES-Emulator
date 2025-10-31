use crate::cpu6502::CPU;

impl CPU {
    pub(crate) fn handleSTY(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let address = _opt_address.expect("BUG: address of STY should be present");
        self.write_u8(address, self.y_register);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_sty_stores_y_register_in_memory() {
        let mut cpu = new_cpu();
        let address = 0x0200;
        cpu.y_register = 0x42;
        let initial_status = cpu.status_register;

        let cycles = cpu.handleSTY(None, Some(address));

        assert_eq!(cycles, 0, "STY should not return extra cycles");
        assert_eq!(cpu.read_u8(address), 0x42, "Y register value should be stored at the address");
        assert_eq!(cpu.status_register, initial_status, "STY should not affect any flags");
    }
}
