use crate::cpu6502::{CPU, StatusFlag};
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handle_cli(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        self.set_status_flag(StatusFlag::InterruptDisable, false);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_cli_clears_interrupt_disable_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // Set carry bit then execute CLC
        cpu.set_status_flag(StatusFlag::InterruptDisable, true);
        let extra = cpu.handle_cli(None, None);
        assert_eq!(cpu.get_status_flag(StatusFlag::InterruptDisable), false);
        assert_eq!(extra, 0);
    }

    #[test]
    fn test_cli_does_not_affect_other_flags() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // Set multiple flags
        cpu.set_status_flag(StatusFlag::InterruptDisable, true);
        cpu.set_status_flag(StatusFlag::Zero, true);
        cpu.set_status_flag(StatusFlag::Negative, true);

        cpu.handle_cli(None, None);

        // Carry cleared, others unchanged
        assert_eq!(cpu.get_status_flag(StatusFlag::InterruptDisable), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Negative), true);
    }
}
