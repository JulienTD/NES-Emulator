const HEADER_SIZE: usize = 16;
const MAGIC_NUMBERS: &[u8; 4] = b"NES\x1a";

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MapperType {
    Nrom = 0,  // Mario, Donkey Kong, etc.
    Mmc1 = 1,  // Zelda, Metroid
    Uxrom = 2, // Castlevania, Mega Man
    Cnrom = 3, // Cybernoid
    Mmc3 = 4,  // Super Mario Bros 3
    Unknown,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

// NES file header structure (16 bytes)
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct NesHeader {
    // The first 4 bytes should be "NES" followed by 0x1A (4E 45 53 1A)
    pub magic_numbers: [u8; 4],
    pub prg_rom_size: u8,
    pub chr_rom_size: u8,
    pub flags_6: u8,
    pub flags_7: u8,
    pub prg_ram_size: u8,
    pub flags_9: u8,
    pub flags_10: u8,
    pub reserved: [u8; 5],
}

// ROM structure to hold NES ROM data
// Parsing is performed by following the header description at this link: (https://formats.kaitai.io/ines/index.html)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct Rom {
    pub header: NesHeader,
    pub mirroring: Mirroring,
    pub mapper: u8,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Option<Vec<u8>>,
}

impl Rom {
    pub(crate) fn parse_nes_rom(rom_data: Vec<u8>) -> Result<Rom, String> {
        if rom_data.len() < HEADER_SIZE {
            return Err(format!("ROM too short: {} bytes (minimum {})", rom_data.len(), HEADER_SIZE));
        }
        if &rom_data[0..4] != MAGIC_NUMBERS {
            return Err("File is not in iNES format".to_string());
        }

        // Parse the iNES header
        let header = NesHeader {
            magic_numbers: [rom_data[0], rom_data[1], rom_data[2], rom_data[3]],
            prg_rom_size: rom_data[4],
            chr_rom_size: rom_data[5],
            flags_6: rom_data[6],
            flags_7: rom_data[7],
            prg_ram_size: rom_data[8],
            flags_9: rom_data[9],
            flags_10: rom_data[10],
            reserved: [rom_data[11], rom_data[12], rom_data[13], rom_data[14], rom_data[15]],
        };

        // Bit 4-7 of Byte 6 are the LOWER 4 bits of the Mapper
        // Bit 4-7 of Byte 7 are the UPPER 4 bits of the Mapper
        let mapper = (header.flags_7 & 0b1111_0000) | (header.flags_6 >> 4);

        // // If true, the game has a Save File (SRAM) at 0x6000
        // let has_battery = (header.flags_6 & 0b0000_0010) != 0;

        // If true, we must skip the first 512 bytes of the ROM input
        let has_trainer = (header.flags_6 & 0b0000_0100) != 0;

        // If true, the cartridge uses four-screen VRAM layout
        let four_screen = (header.flags_6 & 0b0000_1000) != 0;

        // If true, the mirroring is horizontal instead of vertical
        let mirrored = (header.flags_6 & 0b0000_0001) != 0;

        // Bit 0: Mirroring (0=Vertical, 1=Horizontal)
        // Bit 3: Four Screen VRAM
        let mirroring = if four_screen {
            Mirroring::FourScreen
        } else if mirrored {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };

        // Calculate the offset where PRG ROM actually begins.
        // This accounts for the Header (16 bytes) AND the Trainer (512 bytes) if present.
        let prg_rom_start = HEADER_SIZE + if has_trainer { 512 } else { 0 };

        // Calculate the size of the PRG ROM (16KB units)
        let prg_rom_len = header.prg_rom_size as usize * 16384;

        // Determine the end of PRG ROM / start of CHR ROM
        let chr_rom_start = prg_rom_start + prg_rom_len;

        // Calculate the size of CHR ROM (8KB units)
        let chr_rom_len = header.chr_rom_size as usize * 8192;

        let prg_rom_end = prg_rom_start + prg_rom_len;
        let chr_rom_end = chr_rom_start + chr_rom_len;

        if prg_rom_end > rom_data.len() {
            return Err(format!("ROM truncated: PRG ROM ends at {} but file is {} bytes", prg_rom_end, rom_data.len()));
        }
        if chr_rom_len > 0 && chr_rom_end > rom_data.len() {
            return Err(format!("ROM truncated: CHR ROM ends at {} but file is {} bytes", chr_rom_end, rom_data.len()));
        }

        Ok(Rom {
            header,
            prg_rom: rom_data[prg_rom_start..prg_rom_end].to_vec(),
            chr_rom: Some(rom_data[chr_rom_start..chr_rom_end].to_vec()),
            mirroring,
            mapper,
        })
    }

    // Returns the MapperType based on the mapper ID byte.
    pub fn get_mapper_type(&self) -> MapperType {
        match self.mapper {
            0 => MapperType::Nrom,
            1 => MapperType::Mmc1,
            2 => MapperType::Uxrom,
            3 => MapperType::Cnrom,
            4 => MapperType::Mmc3,
            _ => MapperType::Unknown,
        }
    }

    // Performs a sanity check on the ROM to ensure it is playable by this emulator.
    // This function should be called immediately after loading a ROM.
    pub fn check_validity(&self) -> Result<(), String> {
        // Check Magic Number
        if self.header.magic_numbers != *MAGIC_NUMBERS {
             return Err("Invalid ROM: Wrong magic numbers".to_string());
        }

        // Check Mapper Support
        match self.get_mapper_type() {
            MapperType::Nrom => {
                // NROM specific checks:
                // PRG ROM must be either 16KB (1 unit) or 32KB (2 units)
                if self.header.prg_rom_size != 1 && self.header.prg_rom_size != 2 {
                     return Err(format!("Invalid NROM PRG size: {} units (must be 1 or 2)", self.header.prg_rom_size));
                }
            }
            MapperType::Unknown => {
                return Err(format!("Unsupported Mapper: ID {}", self.mapper));
            }
            _ => {
                return Err(format!("Mapper {} ({:?}) is not yet implemented", self.mapper, self.get_mapper_type()));
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn test_rom() -> Rom {
        let header = NesHeader {
            magic_numbers: [0x4E, 0x45, 0x53, 0x1A], // "NES" + EOF
            prg_rom_size: 1,
            chr_rom_size: 1,
            flags_6: 0, // Default (usually implies Horizontal mirroring in simple mappers)
            flags_7: 0,
            prg_ram_size: 0,
            flags_9: 0, // NTSC
            flags_10: 0,
            reserved: [0; 5],
        };

        // PRG ROM is measured in 16KB units (16384 bytes)
        let prg_data = vec![0xEA; 16384]; // Fill with 0xEA (NOP instruction)

        // CHR ROM is measured in 8KB units (8192 bytes)
        let chr_data = vec![0x00; 8192];  // Fill with empty pattern data

        Rom {
            header,
            mirroring: Mirroring::Horizontal, // Common default
            mapper: 0, // Mapper 0 (NROM)
            prg_rom: prg_data,
            chr_rom: Some(chr_data),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: build a minimal valid iNES byte-vector.
    // prg_units: number of 16KB PRG chunks
    // chr_units: number of 8KB CHR chunks
    // flags_6 / flags_7: header flag bytes
    fn build_ines(prg_units: u8, chr_units: u8, flags_6: u8, flags_7: u8) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"NES\x1a");
        data.push(prg_units);
        data.push(chr_units);
        data.push(flags_6);
        data.push(flags_7);
        data.extend_from_slice(&[0u8; 8]); // remaining header bytes

        let prg_len = prg_units as usize * 16384;
        data.extend(std::iter::repeat(0u8).take(prg_len));

        let chr_len = chr_units as usize * 8192;
        data.extend(std::iter::repeat(0u8).take(chr_len));

        data
    }

    // -----------------------------------------------------------------------
    // 1. parse_nes_rom returns Err for a file shorter than 16 bytes
    // -----------------------------------------------------------------------
    #[test]
    fn test_parse_too_short() {
        let short = vec![0u8; 10];
        assert!(Rom::parse_nes_rom(short).is_err());
    }

    // -----------------------------------------------------------------------
    // 2. parse_nes_rom returns Err for wrong magic numbers
    // -----------------------------------------------------------------------
    #[test]
    fn test_parse_wrong_magic() {
        let mut data = build_ines(1, 1, 0, 0);
        // Corrupt the magic bytes
        data[0] = 0x00;
        data[1] = 0x00;
        assert!(Rom::parse_nes_rom(data).is_err());
    }

    // -----------------------------------------------------------------------
    // 3. parse_nes_rom returns Err if the file is truncated
    //    (header says 32KB PRG but file is shorter)
    // -----------------------------------------------------------------------
    #[test]
    fn test_parse_truncated_prg() {
        // Start with a valid header claiming 32KB PRG
        let mut data: Vec<u8> = Vec::new();
        data.extend_from_slice(b"NES\x1a");
        data.push(2); // 2 × 16KB = 32KB PRG
        data.push(0); // no CHR
        data.extend_from_slice(&[0u8; 10]); // rest of header
        // Append only 100 bytes of PRG data instead of 32768
        data.extend(std::iter::repeat(0u8).take(100));

        assert!(Rom::parse_nes_rom(data).is_err());
    }

    // -----------------------------------------------------------------------
    // 4. parse_nes_rom correctly parses a valid 16KB ROM
    // -----------------------------------------------------------------------
    #[test]
    fn test_parse_valid_16kb_rom() {
        let data = build_ines(1, 1, 0, 0);
        let rom = Rom::parse_nes_rom(data).expect("should parse a valid 16KB ROM");
        assert_eq!(rom.prg_rom.len(), 16384);
    }

    // -----------------------------------------------------------------------
    // 5. parse_nes_rom correctly parses a valid 32KB ROM
    // -----------------------------------------------------------------------
    #[test]
    fn test_parse_valid_32kb_rom() {
        let data = build_ines(2, 1, 0, 0);
        let rom = Rom::parse_nes_rom(data).expect("should parse a valid 32KB ROM");
        assert_eq!(rom.prg_rom.len(), 32768);
    }

    // -----------------------------------------------------------------------
    // 6. check_validity returns Err for an unsupported mapper
    // -----------------------------------------------------------------------
    #[test]
    fn test_check_validity_unsupported_mapper() {
        // Mapper 255 (Unknown) → flags_7 upper nibble = 0xF0, flags_6 upper nibble = 0xF0
        // mapper = (flags_7 & 0xF0) | (flags_6 >> 4) → 0xF0 | 0x0F = 0xFF = 255
        let data = build_ines(1, 1, 0xF0, 0xF0);
        let rom = Rom::parse_nes_rom(data).expect("should parse header");
        assert!(rom.check_validity().is_err(), "unknown mapper should be invalid");
    }

    // -----------------------------------------------------------------------
    // 7. check_validity returns Err for invalid NROM PRG size (e.g. 3 units)
    // -----------------------------------------------------------------------
    #[test]
    fn test_check_validity_invalid_nrom_prg_size() {
        // Mapper 0, but prg_rom_size = 3 (neither 1 nor 2)
        let data = build_ines(3, 1, 0, 0);
        let rom = Rom::parse_nes_rom(data).expect("should parse header");
        // mapper byte = 0 → Nrom, but prg_rom_size = 3 is invalid
        assert!(rom.check_validity().is_err(), "NROM with 3 PRG units should be invalid");
    }

    // -----------------------------------------------------------------------
    // 8. get_mapper_type returns MapperType::Nrom for mapper 0
    // -----------------------------------------------------------------------
    #[test]
    fn test_get_mapper_type_nrom() {
        let data = build_ines(1, 1, 0, 0);
        let rom = Rom::parse_nes_rom(data).expect("should parse");
        assert_eq!(rom.get_mapper_type(), MapperType::Nrom);
    }
}
