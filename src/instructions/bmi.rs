use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleBMI(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of BMI should be present");
        self.branch(self.get_status_flag(StatusFlag::Negative), value as i8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_bmi_branch_taken() {
        let mut cpu = new_cpu();
        cpu.program_counter = 0x1000;
        cpu.set_status_flag(StatusFlag::Negative, true); // Set Negative flag
        let cycles = cpu.handleBMI(Some(0x10), None); // Branch forward by 16
        assert_eq!(cpu.program_counter, 0x1010);
        assert_eq!(cycles, 1); // 1 additional cycle for branch taken
    }

    #[test]
    fn test_bmi_branch_not_taken() {
        let mut cpu = new_cpu();
        cpu.program_counter = 0x1000;
        cpu.set_status_flag(StatusFlag::Negative, false); // Clear Negative flag
        let cycles = cpu.handleBMI(Some(0x10), None); // Branch forward by 32 (crosses page)
        assert_eq!(cpu.program_counter, 0x1000); // PC should remain unchanged
        assert_eq!(cycles, 0); // No additional cycles
    }

    #[test]
    fn test_bmi_page_crossing() {
        let mut cpu = new_cpu();
        cpu.program_counter = 0x10F0;
        cpu.set_status_flag(StatusFlag::Negative, true);
        let cycles = cpu.handleBMI(Some(0x20), None);
        assert_eq!(cpu.program_counter, 0x1110);
        assert_eq!(cycles, 2); // 1 for branch taken + 1 for page crossing
    }
}
