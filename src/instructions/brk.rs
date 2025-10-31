use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleBRK(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        // 1. Push Program Counter + 2 to the stack
        // (PC is incremented by 2 to account for the BRK instruction and its padding byte)
        self.push_u16(self.program_counter + 2);

        // 2. Push Status Register with B flag set
        let mut status = self.status_register;
        status |= 1 << (StatusFlag::BreakCommand as u8);
        status |= 1 << (StatusFlag::Unused as u8);
        self.push_u8(status);

        // 3. Set Interrupt Disable flag
        self.set_status_flag(StatusFlag::InterruptDisable, true);

        // 4. Load PC from interrupt vector at 0xFFFE
        // Note: The BRK instruction shares the IRQ vector.
        self.program_counter = self.read_u16(0xFFFE);

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_brk_instruction() {
        let mut cpu = new_cpu();
        cpu.program_counter = 0x8000;
        // Set the interrupt vector at 0xFFFE to point to 0x1234
        cpu.write_u16(0xFFFE, 0x1234);

        cpu.handleBRK(None, None);

        // Check PC jump
        assert_eq!(cpu.program_counter, 0x1234, "PC should jump to the interrupt vector address");
        // Check stack content (LIFO - Last In, First Out)
        // Status was pushed last, so it's popped first.
        assert_eq!(cpu.pop_u8(), 0b0011_0000, "Status with B and U flags set should be popped first");
        assert_eq!(cpu.pop_u16(), 0x8002, "PC+2 should be popped second");
        // Check Interrupt Disable flag
        assert!(cpu.get_status_flag(StatusFlag::InterruptDisable), "Interrupt Disable flag should be set");
    }
}
