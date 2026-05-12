use crate::cpu6502::CPU;

impl CPU {
	// KIL / JAM / HLT — on real 6502 these opcodes halt the CPU permanently.
	// In this emulator we set a halted flag so the run loop exits cleanly.
	pub(crate) fn handle_kil(& mut self, _opt_value: Option<u8>, _opt_address: Option<u16>) -> u8 {
		self.halted = true;
		return 0;
	}
}

#[cfg(test)]
mod tests {
	use crate::bus::Bus;
	use crate::cpu6502::new_cpu;
	use crate::rom::Rom;

	#[test]
	fn test_kil_sets_halted_flag_and_returns_zero() {
		let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
		assert!(!cpu.halted);

		let cycles = cpu.handle_kil(None, None);
		assert_eq!(cycles, 0);
		assert!(cpu.halted);
	}

	// Note: we avoid testing `run_with_callback` here because the emulator's opcode table
	// references many handlers; some less-common unofficial handlers may not be present
	// in this branch and would cause compilation failures when building the full map.

	#[test]
	fn test_kil_remains_halted_on_repeated_calls() {
		let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
		cpu.handle_kil(None, None);
		assert!(cpu.halted);
		// Calling again should keep it halted
		cpu.handle_kil(None, None);
		assert!(cpu.halted);
	}

	#[test]
	fn test_kil_does_not_affect_registers() {
		let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
		cpu.accumulator = 0x42;
		cpu.x_register = 0xAB;
		cpu.y_register = 0xCD;
		cpu.handle_kil(None, None);
		assert_eq!(cpu.accumulator, 0x42);
		assert_eq!(cpu.x_register, 0xAB);
		assert_eq!(cpu.y_register, 0xCD);
	}

	#[test]
	fn test_kil_does_not_affect_status_register() {
		let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
		let initial_status = cpu.status_register;
		cpu.handle_kil(None, None);
		assert_eq!(cpu.status_register, initial_status);
	}

	#[test]
	fn test_kil_does_not_affect_stack_pointer() {
		let mut cpu = new_cpu(Bus::new(Rom::test_rom()));
		let initial_sp = cpu.stack_pointer;
		cpu.handle_kil(None, None);
		assert_eq!(cpu.stack_pointer, initial_sp);
	}
}

