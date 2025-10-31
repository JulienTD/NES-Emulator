use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleBVS(& mut self, opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of BVS should be present");
        self.branch(self.get_status_flag(StatusFlag::Overflow), value as i8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_bvs_branch_taken() {
        let mut cpu = new_cpu();
        cpu.program_counter = 0x1000;
        cpu.set_status_flag(StatusFlag::Overflow, true); // Overflow set// Branch forward by 16
        let cycles = cpu.handleBVS(Some(0x10), None); // Branch forward by 16
        assert_eq!(cpu.program_counter, 0x1010);
        assert_eq!(cycles, 1);
    }

    #[test]
    fn test_bvs_branch_not_taken() {
        let mut cpu = new_cpu();
        cpu.program_counter = 0x1000;
        cpu.set_status_flag(StatusFlag::Overflow, false); // Overflow clear
        let cycles = cpu.handleBVS(Some(0x10), None);
        assert_eq!(cpu.program_counter, 0x1000);
        assert_eq!(cycles, 0);
    }

    #[test]
    fn test_bvs_page_crossing() {
        let mut cpu = new_cpu();
        cpu.program_counter = 0x10F0;
        cpu.set_status_flag(StatusFlag::Overflow, true);
        let cycles = cpu.handleBVS(Some(0x20), None);
        assert_eq!(cpu.program_counter, 0x1110);
        assert_eq!(cycles, 2);
    }
}
