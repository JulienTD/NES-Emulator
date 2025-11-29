use crate::cpu6502::{CPU, StatusFlag};
use crate::bus::Bus;
use crate::rom::Rom;

impl CPU {
	// LAR — AND memory with stack pointer, transfer result to A, X and SP
	// Flags: N, Z
	pub(crate) fn handle_lar(& mut self, opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
		let value = opt_value.expect("BUG: memory value of LAR should be present");

		let result = value & self.stack_pointer;

		self.accumulator = result;
		self.x_register = result;
		self.stack_pointer = result;

		self.set_status_flag(StatusFlag::Zero, result == 0);
		self.set_status_flag(StatusFlag::Negative, (result & 0x80) != 0);

		return 0;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::cpu6502::new_cpu;

	#[test]
	fn test_lar_loads_a_x_and_sp_and_sets_flags() {
		let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
		let mem = 0x80u8; // high bit set
		cpu.stack_pointer = 0xF0; // example stack pointer

		let _ = cpu.handle_lar(Some(mem), None);

		let expected = mem & 0xF0;
		assert_eq!(cpu.accumulator, expected);
		assert_eq!(cpu.x_register, expected);
		assert_eq!(cpu.stack_pointer, expected);
		assert_eq!(cpu.get_status_flag(StatusFlag::Zero), expected == 0);
		assert_eq!(cpu.get_status_flag(StatusFlag::Negative), (expected & 0x80) != 0);
	}

	#[test]
	fn test_lar_zero_flag() {
		let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
		cpu.stack_pointer = 0x00;

		let _ = cpu.handle_lar(Some(0xFF), None);

		// 0xFF & 0x00 == 0
		assert_eq!(cpu.accumulator, 0x00);
		assert!(cpu.get_status_flag(StatusFlag::Zero));
	}

	#[test]
	fn test_lar_negative_flag() {
		let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
		cpu.stack_pointer = 0x80;

		let _ = cpu.handle_lar(Some(0x80), None);

		// & -> 0x80 => negative
		assert_eq!(cpu.accumulator, 0x80);
		assert!(cpu.get_status_flag(StatusFlag::Negative));
	}
}