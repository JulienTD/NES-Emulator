use crate::cpu6502::CPU;
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handle_txs(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        self.stack_pointer = self.x_register;
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

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
}
