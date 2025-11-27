use crate::cpu6502::{CPU, StatusFlag};
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handleCLD(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        self.set_status_flag(StatusFlag::DecimalMode, false);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_cld_clears_decimal_flag() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.set_status_flag(StatusFlag::DecimalMode, true);
        let extra = cpu.handleCLD(None, None);
        assert_eq!(cpu.get_status_flag(StatusFlag::DecimalMode), false);
        assert_eq!(extra, 0);
    }

    #[test]
    fn test_cld_does_not_affect_other_flags() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.set_status_flag(StatusFlag::DecimalMode, true);
        cpu.set_status_flag(StatusFlag::Carry, true);
        cpu.set_status_flag(StatusFlag::Zero, true);

        cpu.handleCLD(None, None);

        assert_eq!(cpu.get_status_flag(StatusFlag::DecimalMode), false);
        assert_eq!(cpu.get_status_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.get_status_flag(StatusFlag::Zero), true);
    }
}
