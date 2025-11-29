use crate::cpu6502::{CPU, StatusFlag};
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    // AXS (also called SBX): A & X, store in X, then X - imm (without borrow)
    // Implement behavior observed: X = (A & X) & imm? Older sources show: X = (A & X) AND operand then X = X - operand
    // We'll implement widely-known AXS behaviour: A & X -> temp, temp - value -> X (affects N,Z,C)
    pub(crate) fn handle_axs(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of AXS should be present");
        let temp = self.accumulator & self.x_register;
        // Subtract immediate from temp without borrow (i.e., temp - value), set carry if temp >= value
        let (result, borrow) = temp.overflowing_sub(value);
        self.x_register = result;

        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, (result & 0x80) != 0);
        // Carry flag is set if no borrow (i.e., temp >= value)
        self.set_status_flag(StatusFlag::Carry, !borrow);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_axs_basic() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0xFF;
        cpu.x_register = 0x10;
        let _ = cpu.handle_axs(Some(0x05), None);
        // temp = 0x10, result = 0x10 - 0x05 = 0x0B
        assert_eq!(cpu.x_register, 0x0B);
        assert!(cpu.get_status_flag(StatusFlag::Carry));
    }
}
