use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    // XAA / ANE – unofficial: A = (A & X) & imm
    pub(crate) fn handle_xaa(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of XAA should be present");
        let result = (self.accumulator & self.x_register) & value;
        self.accumulator = result;

        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, (result & 0x80) != 0);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu6502::{new_cpu};
    use crate::rom::Rom;

    #[test]
    fn test_xaa_combines_a_x_and_operand() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        cpu.accumulator = 0xFF;
        cpu.x_register = 0x0F;
        let _ = cpu.handle_xaa(Some(0xF0), None);
        assert_eq!(cpu.accumulator, 0x00); // 0xFF & 0x0F & 0xF0 == 0x00

        cpu.accumulator = 0xAB;
        cpu.x_register = 0x0B;
        let _ = cpu.handle_xaa(Some(0x0B), None);
        assert_eq!(cpu.accumulator, 0x0B);
    }
}
