use crate::cpu6502::{CPU, StatusFlag};

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
    use crate::bus::Bus;
    use crate::cpu6502::{new_cpu, StatusFlag};
    use crate::rom::Rom;

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

    #[test]
    fn test_axs_zero_flag_when_result_is_zero() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // temp = A & X = 0x10 & 0x10 = 0x10, subtract 0x10 -> 0
        cpu.accumulator = 0x10;
        cpu.x_register = 0x10;
        let _ = cpu.handle_axs(Some(0x10), None);
        assert_eq!(cpu.x_register, 0x00);
        assert!(cpu.get_status_flag(StatusFlag::Zero));
        assert!(cpu.get_status_flag(StatusFlag::Carry)); // no borrow
    }

    #[test]
    fn test_axs_negative_flag_when_result_has_high_bit() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // temp = 0xFF & 0xFF = 0xFF, subtract 0x01 -> 0xFE (negative)
        cpu.accumulator = 0xFF;
        cpu.x_register = 0xFF;
        let _ = cpu.handle_axs(Some(0x01), None);
        assert_eq!(cpu.x_register, 0xFE);
        assert!(cpu.get_status_flag(StatusFlag::Negative));
        assert!(!cpu.get_status_flag(StatusFlag::Zero));
        assert!(cpu.get_status_flag(StatusFlag::Carry)); // 0xFF >= 0x01
    }

    #[test]
    fn test_axs_carry_cleared_on_borrow() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        // temp = 0x01 & 0xFF = 0x01, subtract 0x10 -> borrow, carry cleared
        cpu.accumulator = 0x01;
        cpu.x_register = 0xFF;
        let _ = cpu.handle_axs(Some(0x10), None);
        // 0x01 - 0x10 wraps to 0xF1
        assert_eq!(cpu.x_register, 0xF1);
        assert!(!cpu.get_status_flag(StatusFlag::Carry));
        assert!(cpu.get_status_flag(StatusFlag::Negative));
    }
}
