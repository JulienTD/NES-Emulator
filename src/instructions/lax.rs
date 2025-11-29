use crate::cpu6502::{CPU, StatusFlag};
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    // LAX loads accumulator and X with the memory operand and sets N/Z
    pub(crate) fn handle_lax(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of LAX should be present");
        self.accumulator = value;
        self.x_register = value;

        self.set_status_flag(StatusFlag::Zero, value == 0);
        self.set_status_flag(StatusFlag::Negative, (value & 0x80) != 0);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_lax_loads_accumulator_and_x() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0x00;
        cpu.x_register = 0x00;

        // Simulate immediate/zero page behavior by directly calling handler
        let _ = cpu.handle_lax(Some(0x42), None);
        assert_eq!(cpu.accumulator, 0x42);
        assert_eq!(cpu.x_register, 0x42);
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert!(!cpu.get_status_flag(StatusFlag::Negative));

        let _ = cpu.handle_lax(Some(0x80), None);
        assert_eq!(cpu.accumulator, 0x80);
        assert_eq!(cpu.x_register, 0x80);
        assert!(cpu.get_status_flag(StatusFlag::Negative));

        let _ = cpu.handle_lax(Some(0x00), None);
        assert!(cpu.get_status_flag(StatusFlag::Zero));
    }
}
