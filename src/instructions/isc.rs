use crate::cpu6502::{CPU, StatusFlag};

impl CPU {
    // ISC (ISB): increment memory then SBC (A - M - (1-C))
    pub(crate) fn handle_isc(& mut self, opt_value: Option<u8>, opt_address: Option<u16>) -> u8 {
        let value = opt_value.expect("BUG: memory value of ISC should be present");
        let address = opt_address.expect("BUG: address of ISC should be present");

        let inc_value = value.wrapping_add(1);
        self.write_u8(address, inc_value);

        // SBC: implemented as ADC with inverted operand and carry
        let inverted = !inc_value;
        let carry_in = if self.get_status_flag(StatusFlag::Carry) { 1 } else { 0 };
        let sum = (self.accumulator as u16) + (inverted as u16) + carry_in;
        let result = sum as u8;

        self.set_status_flag(StatusFlag::Carry, sum > 0xFF);
        self.set_status_flag(StatusFlag::Zero, result == 0);
        self.set_status_flag(StatusFlag::Negative, (result & 0x80) != 0);

        let signed_a = self.accumulator as i8;
        let signed_b = inverted as i8;
        let signed_r = result as i8;
        let overflow = (signed_a >= 0 && signed_b >= 0 && signed_r < 0) || (signed_a < 0 && signed_b < 0 && signed_r >= 0);
        self.set_status_flag(StatusFlag::Overflow, overflow);

        self.accumulator = result;
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu6502::{new_cpu, StatusFlag};
    use crate::rom::Rom;

    #[test]
    fn test_isc_increments_memory_and_subtracts() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0200;
        cpu.write_u8(addr, 0x01);
        cpu.accumulator = 0x10;
        cpu.set_status_flag(StatusFlag::Carry, true);

        let _ = cpu.handle_isc(Some(cpu.read_u8(addr)), Some(addr));

        // memory incremented to 2
        assert_eq!(cpu.read_u8(addr), 0x02);
        // SBC equivalent: 0x10 - 0x02 - (1-C) = 0x0E
        assert_eq!(cpu.accumulator, 0x0E);
    }

    #[test]
    fn test_isc_clears_carry_when_borrow() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0210;
        cpu.write_u8(addr, 0x05);
        cpu.accumulator = 0x05;
        cpu.set_status_flag(StatusFlag::Carry, true);

        let _ = cpu.handle_isc(Some(cpu.read_u8(addr)), Some(addr));

        // memory incremented to 6
        assert_eq!(cpu.read_u8(addr), 0x06);
        // 0x05 - 0x06 -> 0xFF ; carry cleared
        assert_eq!(cpu.accumulator, 0xFF);
        assert!(!cpu.get_status_flag(StatusFlag::Carry));
        assert!(cpu.get_status_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_isc_zero_and_carry_with_initial_borrow() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0220;
        cpu.write_u8(addr, 0x05);
        cpu.accumulator = 0x07;
        cpu.set_status_flag(StatusFlag::Carry, false); // will subtract extra 1

        let _ = cpu.handle_isc(Some(cpu.read_u8(addr)), Some(addr));

        // memory incremented to 6
        assert_eq!(cpu.read_u8(addr), 0x06);
        // 0x07 - 0x06 - 1 = 0x00 -> zero set, carry set (no borrow)
        assert_eq!(cpu.accumulator, 0x00);
        assert!(cpu.get_status_flag(StatusFlag::Zero));
        assert!(cpu.get_status_flag(StatusFlag::Carry));
    }

    #[test]
    fn test_isc_overflow_case() {
        let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
        let addr = 0x0300;
        // memory = 0 -> increment -> 1
        cpu.write_u8(addr, 0x00);
        cpu.accumulator = 0x80; // -128 signed
        cpu.set_status_flag(StatusFlag::Carry, true);

        let _ = cpu.handle_isc(Some(cpu.read_u8(addr)), Some(addr));

        // result = 0x80 - 0x01 = 0x7F (127) -> positive while A was negative => overflow
        assert_eq!(cpu.accumulator, 0x7F);
        assert!(cpu.get_status_flag(StatusFlag::Overflow));
    }
}
