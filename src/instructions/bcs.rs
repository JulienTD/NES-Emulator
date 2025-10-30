use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleBCS(& mut self, value: u8) -> u8 {
        self.branch(self.get_status_flag(StatusFlag::Carry), value as i8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_bcs_branch_taken() {
        let mut cpu = new_cpu();
        cpu.program_counter = 0x1000;
        cpu.set_status_flag(StatusFlag::Carry, true); // Set Carry flag
        let cycles = cpu.handleBCS(0x10); // Branch forward by 16
        assert_eq!(cpu.program_counter, 0x1010);
        assert_eq!(cycles, 1); // 1 additional cycle for branch taken
    }

    #[test]
    fn test_bcs_branch_not_taken() {
        let mut cpu = new_cpu();
        cpu.program_counter = 0x1000;
        cpu.set_status_flag(StatusFlag::Carry, false); // Clear Carry flag
        let cycles = cpu.handleBCS(0x10); // Attempt to branch forward by 16
        assert_eq!(cpu.program_counter, 0x1000); // PC should remain unchanged
        assert_eq!(cycles, 0); // No additional cycles
    }

    #[test]
    fn test_bcs_page_crossing() {
        let mut cpu = new_cpu();
        cpu.program_counter = 0x10F0;
        cpu.set_status_flag(StatusFlag::Carry, true); // Set Carry flag
        let cycles = cpu.handleBCS(0x20); // Branch forward by 32 (crosses page)
        assert_eq!(cpu.program_counter, 0x1110);
        assert_eq!(cycles, 2); // 1 for branch taken + 1 for page crossing
    }
}
