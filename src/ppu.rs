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

    vram: [u8; 0x800], // 2KB of VRAM for nametables and attribute tables

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
            vram: [0; 0x800],
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
            self.ppu_addr = self.ppu_addr.wrapping_add(32);
        } else {
            self.ppu_addr = self.ppu_addr.wrapping_add(1);
        }

        match addr {
            0..=0x1FFF => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.chr_rom[addr as usize];
                result
            }
            0x2000..=0x2FFF => {
                let result = self.internal_data_buf;
                let mirrored_addr = self.mirror_vram_addr(addr);
                self.internal_data_buf = self.vram[mirrored_addr as usize];
                result
            }
            0x3000..=0x3EFF => {
                // Mirror of 0x2000–0x2EFF
                let result = self.internal_data_buf;
                let mirrored_addr = self.mirror_vram_addr(addr - 0x1000);
                self.internal_data_buf = self.vram[mirrored_addr as usize];
                result
            }
            0x3F00..=0x3FFF => {
                // Palette read; $3F10/$3F14/$3F18/$3F1C mirror $3F00/$3F04/$3F08/$3F0C
                let idx = palette_addr(addr);
                self.internal_data_buf = self.palette_table[idx];
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

        // Auto-increment ppu_addr after every write (same rule as reads)
        let ctrl = self.ppu_ctrl;
        if self.get_register_flag(&ctrl, ControlFlag::VramAddIncrement as u8) {
            self.ppu_addr = self.ppu_addr.wrapping_add(32);
        } else {
            self.ppu_addr = self.ppu_addr.wrapping_add(1);
        }

        match addr {
            0..=0x1FFF => {
                // CHR ROM is read-only for mapper 0; ignore writes
            }
            0x2000..=0x2FFF => {
                let mirrored_addr = self.mirror_vram_addr(addr);
                self.vram[mirrored_addr as usize] = data;
            }
            0x3000..=0x3EFF => {
                // Mirror of 0x2000–0x2EFF
                let mirrored_addr = self.mirror_vram_addr(addr - 0x1000);
                self.vram[mirrored_addr as usize] = data;
            }
            0x3F00..=0x3FFF => {
                // $3F10/$3F14/$3F18/$3F1C mirror $3F00/$3F04/$3F08/$3F0C
                let idx = palette_addr(addr);
                self.palette_table[idx] = data;
            }
            _ => {
                panic!("PPU write to invalid address {:04x}", addr);
            }
        }
    }

    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let addr = addr & 0x0FFF; // strip the 0x2000 base
        match self.mirroring {
            Mirroring::Vertical => {
                // NT0/NT2 → bank A (0x000–0x3FF), NT1/NT3 → bank B (0x400–0x7FF)
                addr & 0x07FF
            }
            Mirroring::Horizontal => {
                // NT0/NT1 → bank A (0x000–0x3FF), NT2/NT3 → bank B (0x400–0x7FF)
                // Bit 11 of addr selects which physical bank; bits 0–9 are the offset.
                (addr & 0x03FF) | ((addr & 0x0800) >> 1)
            }
            Mirroring::FourScreen => addr,
        }
    }
}

// $3F10/$3F14/$3F18/$3F1C are mirrors of $3F00/$3F04/$3F08/$3F0C respectively.
fn palette_addr(addr: u16) -> usize {
    let idx = (addr - 0x3F00) as usize % 32;
    if idx >= 0x10 && idx % 4 == 0 { idx - 0x10 } else { idx }
}
