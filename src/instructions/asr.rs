use crate::cpu6502::{CPU, StatusFlag};
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handle_asr(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of ASR should be present");
        let mut temp = self.accumulator & value;

        // Set carry from bit0 before shift
        self.set_status_flag(StatusFlag::Carry, (temp & 0x01) != 0);

        // Shift right
        let result = temp >> 1;
        self.accumulator = result;

        // Set Zero and Negative
        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, (result & 0x80) != 0);

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;
    use crate::bus::Bus;
    use crate::rom::Rom;
    use crate::cpu6502::StatusFlag;

    #[test]
    fn test_asr_and_then_lsr() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0b0000_0011;
        let _ = cpu.handle_asr(Some(0b0000_0011), None);
        // temp = 3, shift => 1
        assert_eq!(cpu.accumulator, 0b0000_0001);
        assert!(cpu.get_status_flag(StatusFlag::Carry));
    }
}
