mod cpu6502;
mod instructions;
mod rom;
mod bus;

use crate::cpu6502::trace;
use crate::cpu6502::{CPU};
use crate::cpu6502::new_cpu;
use crate::rom::Rom;
use crate::bus::Bus;


fn main() {
    let rom_data = std::fs::read("./nestest.nes").expect("Failed to read ROM file");
    let rom = Rom::parse_nes_rom(rom_data).expect("Failed to parse ROM");
    rom.check_validity().expect("ROM validity check failed");

    // println!("ROM Loaded successfully!");
    // println!("PRG ROM Size: {} bytes", rom.prg_rom.len());
    // println!("CHR ROM Size: {} bytes", rom.chr_rom.len());
    // println!("Mapper ID: {}", rom.mapper);
    // println!("Mirroring: {:?}", rom.mirroring);
    // println!("Header: {:?}", rom.header);

    let bus = Bus::new(rom);
    let mut cpu: CPU = new_cpu(bus);
    cpu.reset();
    cpu.program_counter = 0xC000;
    cpu.run_with_callback(move |cpu| {
       println!("{}", trace(cpu));
    });

    // cpu.run();

}

