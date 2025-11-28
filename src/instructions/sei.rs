use crate::cpu6502::CPU;
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handle_sei(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        self.set_status_flag(crate::cpu6502::StatusFlag::InterruptDisable, true);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;
    #[test]
    fn test_sei_sets_interrupt_disable_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // Clear decimal mode bit then execute SEI
        cpu.set_status_flag(crate::cpu6502::StatusFlag::InterruptDisable, false);
        let extra = cpu.handle_sei(None, None);
        assert_eq!(cpu.get_status_flag(crate::cpu6502::StatusFlag::InterruptDisable), true);
        assert_eq!(extra, 0);
    }
    #[test]
    fn test_sei_does_not_affect_other_flags() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // Set multiple flags
        cpu.set_status_flag(crate::cpu6502::StatusFlag::InterruptDisable, false);
        cpu.set_status_flag(crate::cpu6502::StatusFlag::Zero, true);
        cpu.set_status_flag(crate::cpu6502::StatusFlag::Negative, true);
        cpu.set_status_flag(crate::cpu6502::StatusFlag::Carry, true);
        cpu.set_status_flag(crate::cpu6502::StatusFlag::DecimalMode, true);

        cpu.handle_sei(None, None);

        // Decimal mode set, others unchanged
        assert_eq!(cpu.get_status_flag(crate::cpu6502::StatusFlag::InterruptDisable), true);
        assert_eq!(cpu.get_status_flag(crate::cpu6502::StatusFlag::Zero), true);
        assert_eq!(cpu.get_status_flag(crate::cpu6502::StatusFlag::Negative), true);
        assert_eq!(cpu.get_status_flag(crate::cpu6502::StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(crate::cpu6502::StatusFlag::DecimalMode), true);
    }
}
