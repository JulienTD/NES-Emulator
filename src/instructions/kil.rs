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
}

