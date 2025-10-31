use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleBEQ(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of BEQ should be present");
        self.branch(self.get_status_flag(StatusFlag::Zero), value as i8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_beq_branch_taken() {
        let mut cpu = new_cpu();
        cpu.program_counter = 0x1000;
        cpu.set_status_flag(StatusFlag::Zero, true); // Set Zero flag
        let cycles = cpu.handleBEQ(Some(0x10), None); // Branch forward by 16
        assert_eq!(cpu.program_counter, 0x1010);
        assert_eq!(cycles, 1); // 1 additional cycle for branch taken
    }

    #[test]
    fn test_beq_branch_not_taken() {
        let mut cpu = new_cpu();
        cpu.program_counter = 0x1000;
        cpu.set_status_flag(StatusFlag::Zero, false); // Clear Zero flag
        let cycles = cpu.handleBEQ(Some(0x10), None); // Attempt to branch forward by 16
        assert_eq!(cpu.program_counter, 0x1000); // PC should remain unchanged
        assert_eq!(cycles, 0); // No additional cycles
    }

    #[test]
    fn test_beq_page_crossing() {
        let mut cpu = new_cpu();
        cpu.program_counter = 0x10F0;
        cpu.set_status_flag(StatusFlag::Zero, true); // Set Zero flag
        let cycles = cpu.handleBEQ(Some(0x20), None); // Branch forward by 32 (crosses page)
        assert_eq!(cpu.program_counter, 0x1110);
        assert_eq!(cycles, 2); // 1 for branch taken + 1 for page crossing
    }
}
