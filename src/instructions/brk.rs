use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleBRK(& mut self, opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        // // Set the Break flag before pushing status
        // self.set_status_flag(StatusFlag::Break, true);
        // self.push_u16(self.program_counter + 1); // Push return address (PC + 1)
        // self.push_u8(self.get_status_byte()); // Push status register
        // self.set_status_flag(StatusFlag::InterruptDisable, true); // Set Interrupt Disable flag
        // // Load the interrupt vector at $FFFE/$FFFF
        // let lo = self.read_u8(0xFFFE) as u16;
        // let hi = self.read_u8(0xFFFF) as u16;
        // self.program_counter = (hi << 8) | lo;
        // 0 // BRK does not add extra cycles beyond the base
        return 0;
    }
}
