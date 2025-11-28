const HEADER_SIZE: usize = 16;
const MAGIC_NUMBERS: &[u8; 4] = b"NES\x1a";

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

// NES file header structure (16 bytes)
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
#[derive(Debug, Clone)]
pub(crate) struct Rom {
    pub header: NesHeader,
    pub mirroring: Mirroring,
    pub mapper: u8,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}

impl Rom {
    pub(crate) fn parse_nes_rom(rom_data: Vec<u8>) -> Result<Rom, String> {
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

        // Determine where the actual ROM data starts
        // The header is 16 bytes.
        let prg_rom_start = HEADER_SIZE + if has_trainer {
            512 // Skip "Trainer" data if bit 2 is set
        } else {
            0
        };

        // Slice the PRG ROM (Program Code)
        // let prg_rom_len: usize = header.prg_rom_size as usize * 16384; // 16KB units
        // let prg_rom = rom_data[prg_rom_start .. (prg_rom_start + prg_rom_len)].to_vec();

        // // Slice the CHR ROM (Graphics)
        // let chr_rom_start = prg_rom_start + prg_rom_len;
        // let chr_rom_len = header.chr_rom_size as usize * 8192; // 8KB units
        // let chr_rom = rom_data[chr_rom_start .. (chr_rom_start + chr_rom_len)].to_vec();

        return Ok(Rom {
            header,
            prg_rom: rom_data[HEADER_SIZE..(HEADER_SIZE + (header.prg_rom_size as usize * 16384))].to_vec(),
            chr_rom: rom_data[(HEADER_SIZE + (header.prg_rom_size as usize * 16384))..].to_vec(),
            mirroring,
            mapper,
        });
    }

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
            chr_rom: chr_data,
        }
    }
}
