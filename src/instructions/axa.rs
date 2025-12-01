use crate::cpu6502::{CPU};

impl CPU {
    pub(crate) fn handle_axa(& mut self, _opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let address = opt_address.expect("BUG: address of AXA should be present");

        // AXA/AAX: store (A & X & (high_byte(address) + 1)) into memory
        let high = (address >> 8) as u8;
        let result = self.accumulator & self.x_register & high.wrapping_add(1);
        self.write_u8(address, result);
        return 0;
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::bus::Bus;
//     use crate::cpu6502::{new_cpu};
//     use crate::rom::Rom;

//     #[test]
//     fn test_axa_stores_and_of_a_x_and_high_plus_one() {
//         let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
//         cpu.accumulator = 0xF0;
//         cpu.x_register = 0x0F;
//         let addr = 0x0200; // high byte = 0x02

//         let cycles = cpu.handle_axa(None, Some(addr));
//         assert_eq!(cycles, 0);
//         // 0xF0 & 0x0F & (0x02+1) == 0x00
//         assert_eq!(cpu.read_u8(addr), 0x00);

//         cpu.accumulator = 0xAB;
//         cpu.x_register = 0x0B;
//         let _ = cpu.handle_axa(None, Some(addr));
//         // 0xAB & 0x0B & (0x02+1) == 0x0B
//         assert_eq!(cpu.read_u8(addr), 0x0B);
//     }
// }
