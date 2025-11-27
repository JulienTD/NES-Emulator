use crate::rom::Rom;

// The 6502 has a 16 bit address bus, which means it can address up to 64KB of memory.
// This memory is typically divided into several regions, including RAM, ROM, and memory-mapped I/O.
// Memory map:
// 0x0000 - 0x1FFF: RAM (mirrored every 0x0800 bytes)
// 0x2000 - 0x3FFF: PPU Registers (mirrored every 8 bytes)
// 0x4000 - 0x401F: APU and I/O Registers
// 0x4020 - 0x5FFF: Expansion ROM
// 0x6000 - 0x7FFF: Save RAM
// 0x8000 - 0xFFFF: PRG ROM
// Total memory size: 64KB; 0xFFFF + 1 = 65536 bytes = 0x10000 to include all addresses.

#[derive(Debug)]
pub(crate) struct Bus {
    internal_ram: [u8; 0x0800], // 2KB internal RAM (0x0000 - 0x07FF)
    rom: Rom,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        Self {
            internal_ram: [0; 0x0800],
            rom
        }
    }

    pub fn read_u8(&self, mut addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let mirrored_addr = addr & 0x07FF; // Mirroring logic for 2KB RAM
                self.internal_ram[mirrored_addr as usize]
            }
            0x2000..=0x3FFF => {
                todo!("PPU is not supported yet")
            }
            0x8000..=0xFFFF => {
                addr -= 0x8000;
                if self.rom.prg_rom.len() == 0x4000 && addr >= 0x4000 {
                    //mirror if needed
                    addr = addr % 0x4000;
                }
                self.rom.prg_rom[addr as usize]
            }
            _ => {
                println!("Memory access at {} not handled", addr);
                // Handle other address ranges (e.g., APU, Cartridge)
                0 // Placeholder
            }
        }
    }

    pub fn write_u8(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => {
                let mirrored_addr = addr & 0x07FF; // Mirroring logic for 2KB RAM
                self.internal_ram[mirrored_addr as usize] = data;
            }
            0x2000..=0x3FFF => {
                todo!("PPU is not supported yet")
            }
            0x8000..=0xFFFF => {
                // PRG ROM is not writable. Ignore writes or log a warning.
                println!("Attempted write to PRG ROM at address {:04X}", addr);
            }
            _ => {
                println!("Memory access at {} not handled", addr);
                // Handle other address ranges (e.g., APU, Cartridge)
            }
        }
    }
}
