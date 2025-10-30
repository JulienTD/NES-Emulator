use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    pub(crate) fn handleASL(& mut self, value: u8) -> u8 {
        let result = value << 1;

        // Set Carry flag (C) - set if bit 7 of original value is set
        self.set_status_flag(StatusFlag::Carry, (value & 0x80) != 0);

        // Set Zero flag (Z) - set if result = 0
        self.set_status_flag(StatusFlag::Zero, result == 0);

        // Set Negative flag (N) - set if bit 7 of result is set
        self.set_status_flag(StatusFlag::Negative, (result & 0x80) != 0);

        self.accumulator = result;
        return 0;
    }
}
