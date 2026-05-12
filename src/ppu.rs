use crate::rom::Mirroring;

#[derive(Debug)]
pub(crate) struct PPU {
    // PPU registers
    /// NMI enable (V), PPU master/slave (P), sprite height (H), background tile select (B), sprite tile select (S), increment mode (I), nametable select / X and Y scroll bit 8 (NN)
    /// VPHB SINN
    pub ppu_ctrl: u8,

    /// color emphasis (BGR), sprite enable (s), background enable (b), sprite left column enable (M), background left column enable (m), greyscale (G)
    /// BGRs bMmG
    pub ppu_mask: u8,

    /// vblank (V), sprite 0 hit (S), sprite overflow (O); read resets write pair for $2005/$2006
    /// VSO- ----
    pub ppu_status: u8,

    /// X and Y scroll bits 7-0 (two writes: X scroll, then Y scroll)
    /// XXXX XXXX YYYY YYYY
    pub ppu_scroll: u8,

    /// VRAM address (two writes: most significant byte, then least significant byte)
    pub ppu_addr: u16,

    /// VRAM data read/write
    pub ppu_data: u8,

    /// OAM read/write address
    pub oam_addr: u8,

    /// OAM data read/write
    pub oam_data: [u8; 256], // Object Attribute Memory (OAM)

    /// OAM DMA high address
    pub oam_dma: u8,

    // PPU Internal memory and state
    chr_rom: Vec<u8>, // Character ROM (CHR ROM) from the cartridge

    vram: [u8; 0x400], // 1KB of VRAM for nametables and attribute tables

    palette_table: [u8; 32], // 32 bytes for background and sprite palettes

    mirroring: Mirroring, // Mirroring type for nametable addressing

    /// Internal states
    ppu_addr_latch: bool, // Latch for tracking first/second write to $2005/$2006
    internal_data_buf: u8,

    scanline: u16,
    cycles: usize,
}

// Each flag corresponds to a bit in the control register
// Values are the bit positions
#[derive(Debug, Clone, Copy)]
pub(crate) enum ControlFlag {
    Nametable1 = 0,
    Nametable2 = 1,
    VramAddIncrement = 2,
    SpritePatternTable = 3,
    BackgroundPatternTable = 4,
    SpriteSize = 5,
    MasterSlave = 6,
    GenerateNMI = 7,
}

// Each flag corresponds to a bit in the control register
// Values are the bit positions
#[derive(Debug, Clone, Copy)]
pub(crate) enum MaskFlag {
    Greyscale = 0,
    ShowBackgroundLeftmost = 1,
    ShowSpritesLeftmost = 2,
    ShowBackground = 3,
    ShowSprites = 4,
    EmphasizeRed = 5,
    EmphasizeGreen = 6,
    EmphasizeBlue = 7,
}

// Each flag corresponds to a bit in the status register
// Values are the bit positions
#[derive(Debug, Clone, Copy)]
pub(crate) enum StatusFlag {
    SpriteOverflow = 5,
    Sprite0Hit = 6,
    Vblank = 7,
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        PPU {
            ppu_ctrl: 0,
            ppu_mask: 0,
            ppu_status: 0,
            ppu_scroll: 0,
            ppu_addr: 0,
            ppu_data: 0,
            oam_addr: 0,
            oam_data: [0; 256],
            oam_dma: 0,
            chr_rom,
            vram: [0; 0x400],
            palette_table: [0; 32],
            mirroring,
            ppu_addr_latch: false,
            internal_data_buf: 0,
            scanline: 0,
            cycles: 0,
        }
    }

    pub(crate) fn set_register_flag(& mut self, register: &mut u8, flag: u8, value: bool) {
        if value {
            *register |= 1 << flag;
        } else {
            *register &= !(1 << flag);
        }
    }

    pub(crate) fn get_register_flag(&self, register: &u8, flag: u8) -> bool {
        (*register & (1 << flag)) != 0
    }

    pub fn write_ppu_address(&mut self, data: u8) {
        if !self.ppu_addr_latch {
            // First write (most significant byte)
            self.ppu_addr = (data as u16) << 8;
            self.ppu_addr_latch = true;
        } else {
            // Second write (least significant byte)
            self.ppu_addr |= data as u16;
            self.ppu_addr_latch = false;
        }
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.ppu_addr;

        // After reading, the PPU address should increment by either 1 or 32 depending on the increment mode set in PPUCTRL.
        // if self.ppu_ctrl & 0b00000100 == 0 {
        let ctrl = self.ppu_ctrl;
        if self.get_register_flag(&ctrl, ControlFlag::VramAddIncrement as u8) {
            self.ppu_addr = self.ppu_addr.wrapping_add(1);
        } else {
            self.ppu_addr = self.ppu_addr.wrapping_add(32);
        }

        match addr {
            0..=0x1FFF => {
                // CHR ROM read
                let result = self.internal_data_buf;
                self.internal_data_buf = self.chr_rom[addr as usize];
                result
            }
            0x2000..=0x2FFF => {
                // VRAM read (nametables)
                let result = self.internal_data_buf;
                let mirrored_addr = self.mirror_vram_addr(addr);
                self.internal_data_buf = self.vram[mirrored_addr as usize];
                result
            }
            0x3F00..=0x3FFF => {
                // Palette read
                self.internal_data_buf = self.palette_table[((addr - 0x3F00) as usize) % 32];
                self.internal_data_buf
            }
            _ => {
                panic!("PPU read from invalid address {:04x}", addr);
            }
        }
    }

    pub fn oam_data(&self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    pub fn read_status(&mut self) -> u8 {
        let status = self.ppu_status;
        // Clear vblank flag on read
        self.ppu_status &= 0b0111_1111;
        // Reset address latch
        self.ppu_addr_latch = false;
        status
    }

    pub fn write_to_data(&mut self, data: u8) {
        let addr = self.ppu_addr;
        match addr {
            0..=0x1FFF => {
                // CHR ROM is typically read-only, but some cartridges have CHR RAM. For simplicity, we'll ignore writes to CHR ROM.
                // If we wanted to support CHR RAM, we would write to a separate CHR RAM array instead of the CHR ROM vector.
                // self.chr_rom[addr as usize] = data; // Not supported for CHR ROM
            }
            0x2000..=0x2FFF => {
                // VRAM write (nametables)
                let mirrored_addr = self.mirror_vram_addr(addr);
                self.vram[mirrored_addr as usize] = data;
            }
            0x3F00..=0x3FFF => {
                // Palette write
                self.palette_table[((addr - 0x3F00) as usize) % 32] = data;
            }
            _ => {
                panic!("PPU write to invalid address {:04x}", addr);
            }
        }
    }

    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        // VRAM address space: 0x2000-0x2FFF is 4KB, but actual VRAM is only 1KB (0x400 bytes)
        // Apply mirroring based on the cartridge's mirroring mode
        let addr = addr & 0x0FFF; // Mask to 0x2000-0x2FFF range
        match self.mirroring {
            Mirroring::Vertical => {
                // Vertical mirroring: nametables at 0x2000 and 0x2800 map to same VRAM
                addr & 0x07FF
            }
            Mirroring::Horizontal => {
                // Horizontal mirroring: nametables at 0x2000 and 0x2400 map to same VRAM
                (addr >> 1) & 0x03FF
            }
            Mirroring::FourScreen => {
                // Four-screen: no mirroring, 4KB of VRAM (not supported with 1KB VRAM)
                addr
            }
        }
    }

    //     pub(crate) fn set_status_flag(& mut self, flag: StatusFlag, value: bool) {
    //     if value {
    //         self.status_register |= 1 << (flag as u8);
    //     } else {
    //         self.status_register &= !(1 << (flag as u8));
    //     }
    // }

    // pub(crate) fn get_status_flag(&self, flag: StatusFlag) -> bool {
    //     (self.status_register & (1 << (flag as u8))) != 0
    // }

    //     pub fn tick(&mut self, cycles: u8) -> bool {
    //        self.cycles += cycles as usize;
    //        if self.cycles >= 341 {
    //            self.cycles = self.cycles - 341;
    //            self.scanline += 1;

    //            if self.scanline == 241 {
    //                if self.ctrl.generate_vblank_nmi() {
    //                    self.status.set_vblank_status(true);
    //                    todo!("Should trigger NMI interrupt")
    //                }
    //            }

    //            if self.scanline >= 262 {
    //                self.scanline = 0;
    //                self.status.reset_vblank_status();
    //                return true;
    //            }
    //        }
    //        return false;
    //    }
}
