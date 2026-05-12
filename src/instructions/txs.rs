use crate::cpu6502::CPU;

impl CPU {
    pub(crate) fn handle_txs(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        self.stack_pointer = self.x_register;
        return 0;
    }
}

#[cfg(test)]
mod tests {

    use crate::bus::Bus;
    use crate::cpu6502::new_cpu;
    use crate::rom::Rom;

    #[test]
    fn test_txs_transfers_x_to_stack_pointer() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.x_register = 0xAB;
        let initial_status = cpu.status_register;

        let cycles = cpu.handle_txs(None, None);

        assert_eq!(cycles, 0, "TXS should not return extra cycles");
        assert_eq!(cpu.stack_pointer, 0xAB, "Stack pointer should be set to the value of X register");
        assert_eq!(cpu.status_register, initial_status, "TXS should not affect any flags");
    }

    #[test]
    fn test_txs_transfers_zero_from_x() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.x_register = 0x00;
        cpu.handle_txs(None, None);
        assert_eq!(cpu.stack_pointer, 0x00);
    }

    #[test]
    fn test_txs_transfers_0xff_from_x() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.x_register = 0xFF;
        cpu.handle_txs(None, None);
        assert_eq!(cpu.stack_pointer, 0xFF);
    }

    #[test]
    fn test_txs_does_not_change_x_register() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.x_register = 0x77;
        cpu.handle_txs(None, None);
        assert_eq!(cpu.x_register, 0x77, "TXS should not modify X register");
    }

    #[test]
    fn test_txs_does_not_change_accumulator_or_y_register() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x12;
        cpu.y_register = 0x34;
        cpu.x_register = 0x56;
        cpu.handle_txs(None, None);
        assert_eq!(cpu.accumulator, 0x12);
        assert_eq!(cpu.y_register, 0x34);
    }
}
