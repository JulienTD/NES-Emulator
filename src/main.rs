mod cpu6502;
mod instructions;
use crate::cpu6502::{CPU, StatusFlag};
use crate::cpu6502::new_cpu;

fn main() {
    let mut cpu: CPU = new_cpu();
    cpu.reset();
}
