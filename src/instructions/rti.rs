use crate::cpu6502::{CPU, StatusFlag};
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handleRTI(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
        let popped_status = self.pop_u8();
        self.program_counter = self.pop_u16();

        // The B and U flags are not affected by RTI.
        // We need to preserve their current state from the status register.
        let b_flag_mask = 1 << (StatusFlag::BreakCommand as u8);
        let u_flag_mask = 1 << (StatusFlag::Unused as u8);
        let preserved_mask = b_flag_mask | u_flag_mask;

        self.status_register = (popped_status & !preserved_mask) | (self.status_register & preserved_mask);

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;

    #[test]
    fn test_rti_restores_status_and_pc() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let return_address = 0x1234;
        let status_on_stack = 0b1011_0101; // A status with B and U flags set

        // Simulate an interrupt by pushing PC and status
        cpu.push_u16(return_address);
        cpu.push_u8(status_on_stack);

        cpu.handleRTI(None, None);

        assert_eq!(cpu.program_counter, return_address, "Program counter should be restored");
        // The status register should be 0b1000_0101. C, Z, and N are set from the stack,
        // but B and U are ignored and retain their original (0) value.
        assert_eq!(cpu.status_register, 0b1000_0101, "Status register should be restored, ignoring B and U flags");
        assert_eq!(cpu.stack_pointer, 0xFF, "Stack pointer should be restored to its original state");
    }
}
