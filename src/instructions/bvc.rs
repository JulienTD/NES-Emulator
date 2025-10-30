use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleBVC(& mut self, value: u8) -> u8 {
        self.branch(!self.get_status_flag(StatusFlag::Overflow), value as i8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_bvc_branch_taken() {
        let mut cpu = new_cpu();
        cpu.program_counter = 0x1000;
        cpu.set_status_flag(StatusFlag::Overflow, false); // Overflow clear
        let cycles = cpu.handleBVC(0x10); // Branch forward by 16
        assert_eq!(cpu.program_counter, 0x1010);
        assert_eq!(cycles, 1);
    }

    #[test]
    fn test_bvc_branch_not_taken() {
        let mut cpu = new_cpu();
        cpu.program_counter = 0x1000;
        cpu.set_status_flag(StatusFlag::Overflow, true); // Overflow set
        let cycles = cpu.handleBVC(0x10);
        assert_eq!(cpu.program_counter, 0x1000);
        assert_eq!(cycles, 0);
    }

    #[test]
    fn test_bvc_page_crossing() {
        let mut cpu = new_cpu();
        cpu.program_counter = 0x10F0;
        cpu.set_status_flag(StatusFlag::Overflow, false);
        let cycles = cpu.handleBVC(0x20);
        assert_eq!(cpu.program_counter, 0x1110);
        assert_eq!(cycles, 2);
    }
}
