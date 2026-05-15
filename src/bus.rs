use crate::rom::Rom;
use crate::ppu::PPU;

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

pub(crate) struct Bus {
    internal_ram: [u8; 0x0800], // 2KB internal RAM (0x0000 - 0x07FF)
    rom: Rom,
    ppu: PPU,
    cycles: usize,
    nmi_callback: Option<Box<dyn FnMut(&PPU)>>,
}

impl std::fmt::Debug for Bus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bus")
            .field("cycles", &self.cycles)
            .field("ppu", &self.ppu)
            .finish()
    }
}

impl Bus {
    pub(crate) fn new(mut rom: Rom) -> Self {
        let chr_rom = rom.chr_rom.take().unwrap();
        let mirroring = rom.mirroring.clone();

        Self {
            internal_ram: [0; 0x0800],
            rom,
            ppu: PPU::new(chr_rom, mirroring),
            cycles: 0,
            nmi_callback: None,
        }
    }

    pub fn set_nmi_callback(&mut self, callback: impl FnMut(&PPU) + 'static) {
        self.nmi_callback = Some(Box::new(callback));
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;
        let nmi_before = self.ppu.nmi_interrupt.is_some();
        self.ppu.tick(cycles * 3);
        let nmi_after = self.ppu.nmi_interrupt.is_some();
        if !nmi_before && nmi_after {
            if let Some(mut cb) = self.nmi_callback.take() {
                cb(&self.ppu);
                self.nmi_callback = Some(cb);
            }
        }
    }

    pub fn poll_nmi_status(&mut self) -> Option<u8> {
        self.ppu.nmi_interrupt.take()
    }

    pub fn read_u8(&mut self, mut addr: u16) -> u8 {
        match addr {
            // RAM (0x0000 - 0x1FFF)
            // The 2KB RAM is mirrored 4 times. Reading 0x0000 is the same as 0x0800.
            0x0000..=0x1FFF => {
                let mirrored_addr = addr & 0x07FF; // Mirroring logic for 2KB RAM
                self.internal_ram[mirrored_addr as usize]
            }

            // PPU Registers (0x2000 - 0x3FFF)

            0x2000 => {
                self.ppu.ppu_ctrl
            }

            // 0x2000..=0x3FFF => {
            //     let _mirror_down_addr = addr & 0b0010_0000_0000_0111;
            //     todo!("PPU is not supported yet")
            // }

            0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                0 // write-only registers return open-bus on real hardware
            }
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.oam_data(),
            0x2007 => self.ppu.read_data(),

            0x2008..=0x3FFF => {
                let mirror_down_addr = addr & 0x2007;
                self.read_u8(mirror_down_addr)
            }


            // Cartridge Space (0x8000 - 0xFFFF)
            0x8000..=0xFFFF => {
                // Shift address down so 0x8000 becomes 0x0000
                addr -= 0x8000;

                // Mapper 0 (NROM) Logic:
                // If PRG ROM is 16KB (len = 16384), it is mirrored.
                // The CPU expects code at 0xC000, but we only have data up to 0x4000.
                // So we mirror 0xC000-0xFFFF back to 0x8000-0xBFFF.
                if self.rom.prg_rom.len() == 16384 && addr >= 16384 {
                    addr = addr % 16384;
                }
                self.rom.prg_rom[addr as usize]
            }

            // APU / I-O registers — not yet implemented; return open-bus value
            0x4000..=0x401F => 0xFF,

            _ => {
                println!("Memory access at {} not handled", addr);
                0
            }
        }
    }

    pub fn write_u8(&mut self, addr: u16, data: u8) {
        match addr {
            // RAM
            0x0000..=0x1FFF => {
                let mirrored_addr = addr & 0x07FF; // Mirroring logic for 2KB RAM
                self.internal_ram[mirrored_addr as usize] = data;
            }

            0x2000 => {
                self.ppu.write_to_ctrl(data);
                // self.ppu.ppu_ctrl = data;
            }

            0x2001 => {
                // self.ppu.write_to_mask(data);
                self.ppu.ppu_mask = data;
            }

            0x2003 => {
                // self.ppu.write_to_oam_addr(data);
                self.ppu.oam_addr = data;
            }

            0x2004 => {
                // self.ppu.write_to_oam_data(data);
                self.ppu.oam_data[self.ppu.oam_addr as usize] = data;
            }

            0x2005 => {
                // self.ppu.write_to_scroll(data);
                self.ppu.ppu_scroll = data;
            }

            0x2006 => {
                // self.ppu.write_to_ppu_addr(data);
                self.ppu.write_ppu_address(data);
            }
            0x2007 => {
                self.ppu.write_to_data(data);

                // self.ppu.write_to_data(data);
            }

            0x2008..=0x3FFF => {
                let mirror_down_addr = addr & 0x2007;
                self.write_u8(mirror_down_addr, data);
            }

            // Cartridge Space
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom::Rom;

    // Helper: create a Bus backed by the standard test ROM (16KB PRG, mapper 0).
    fn make_bus() -> Bus {
        Bus::new(Rom::test_rom())
    }

    // Helper: build a minimal iNES ROM byte-vector from scratch.
    // prg_units: number of 16KB PRG chunks
    // chr_units: number of 8KB CHR chunks
    // flags_6 / flags_7: header flag bytes
    fn build_ines_rom(prg_units: u8, chr_units: u8, flags_6: u8, flags_7: u8, prg_fill: u8) -> Vec<u8> {
        let mut data = Vec::new();
        // Magic number
        data.extend_from_slice(b"NES\x1a");
        // PRG / CHR sizes
        data.push(prg_units);
        data.push(chr_units);
        // Flags
        data.push(flags_6);
        data.push(flags_7);
        // 8 bytes padding to complete 16-byte header
        data.extend_from_slice(&[0u8; 8]);

        // PRG data
        let prg_len = prg_units as usize * 16384;
        data.extend(std::iter::repeat(prg_fill).take(prg_len));

        // CHR data
        let chr_len = chr_units as usize * 8192;
        data.extend(std::iter::repeat(0u8).take(chr_len));

        data
    }

    // -----------------------------------------------------------------------
    // 1. RAM read/write at base address
    // -----------------------------------------------------------------------
    #[test]
    fn test_ram_read_write_base() {
        let mut bus = make_bus();
        bus.write_u8(0x0000, 0xAB);
        assert_eq!(bus.read_u8(0x0000), 0xAB);
    }

    // -----------------------------------------------------------------------
    // 2. RAM mirroring: write 0x0000, read back via 0x0800 / 0x1000 / 0x1800
    // -----------------------------------------------------------------------
    #[test]
    fn test_ram_mirroring() {
        let mut bus = make_bus();
        bus.write_u8(0x0000, 0x42);
        assert_eq!(bus.read_u8(0x0800), 0x42, "mirror at 0x0800");
        assert_eq!(bus.read_u8(0x1000), 0x42, "mirror at 0x1000");
        assert_eq!(bus.read_u8(0x1800), 0x42, "mirror at 0x1800");
    }

    // -----------------------------------------------------------------------
    // 3. PRG ROM read at 0x8000 for a 32KB ROM
    // -----------------------------------------------------------------------
    #[test]
    fn test_prg_rom_read_32kb() {
        // Build a 32KB ROM filled with a distinctive byte value.
        let rom_data = build_ines_rom(2, 1, 0, 0, 0xCC);
        let rom = Rom::parse_nes_rom(rom_data).expect("valid 32KB ROM");
        let mut bus = Bus::new(rom);

        // The first PRG byte should be the fill value.
        assert_eq!(bus.read_u8(0x8000), 0xCC);
    }

    // -----------------------------------------------------------------------
    // 4. PRG ROM mirroring: for a 16KB ROM, 0x8000 and 0xC000 return the same value
    // -----------------------------------------------------------------------
    #[test]
    fn test_prg_rom_mirroring_16kb() {
        // test_rom() is already 16KB (prg_rom_size = 1, filled with 0xEA).
        let mut bus = make_bus();
        let lo = bus.read_u8(0x8000);
        let hi = bus.read_u8(0xC000);
        assert_eq!(lo, hi, "16KB PRG ROM should be mirrored at 0xC000");
        assert_eq!(lo, 0xEA); // test_rom fills with NOPs
    }

    // -----------------------------------------------------------------------
    // 5. APU range (0x4000–0x401F) read returns 0xFF
    // -----------------------------------------------------------------------
    #[test]
    fn test_apu_range_returns_open_bus() {
        let mut bus = make_bus();
        assert_eq!(bus.read_u8(0x4000), 0xFF);
        assert_eq!(bus.read_u8(0x4010), 0xFF);
        assert_eq!(bus.read_u8(0x401F), 0xFF);
    }

    // -----------------------------------------------------------------------
    // 6. Writing to PRG ROM is silently ignored; read back returns original value
    // -----------------------------------------------------------------------
    #[test]
    fn test_write_to_prg_rom_is_ignored() {
        let mut bus = make_bus();
        let original = bus.read_u8(0x8000); // 0xEA from test_rom
        bus.write_u8(0x8000, 0x00);         // attempt to overwrite
        assert_eq!(bus.read_u8(0x8000), original, "write to PRG ROM must be ignored");
    }
}
