use crate::cpu6502::{CPU, StatusFlag};
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
    pub(crate) fn handle_axa(& mut self, opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of AXA should be present");
        let address = opt_address.expect("BUG: address of AXA should be present");

        // AXA/AAX: store (A & X & (high_byte(address) + 1)) into memory
        let high = (address >> 8) as u8;
        let result = self.accumulator & self.x_register & high.wrapping_add(1);
        self.write_u8(address, result);
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu6502::new_cpu;
    use crate::bus::Bus;
    use crate::rom::Rom;
    use crate::cpu6502::StatusFlag;


}
