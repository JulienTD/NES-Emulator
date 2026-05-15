use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use crate::ppu::PPU;
use crate::ppu::ControlFlag;
use crate::ppu::get_register_flag;

pub static SYSTEM_PALLETE: [(u8,u8,u8); 64] = [
   (0x80, 0x80, 0x80), (0x00, 0x3D, 0xA6), (0x00, 0x12, 0xB0), (0x44, 0x00, 0x96), (0xA1, 0x00, 0x5E),
   (0xC7, 0x00, 0x28), (0xBA, 0x06, 0x00), (0x8C, 0x17, 0x00), (0x5C, 0x2F, 0x00), (0x10, 0x45, 0x00),
   (0x05, 0x4A, 0x00), (0x00, 0x47, 0x2E), (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05),
   (0x05, 0x05, 0x05), (0xC7, 0xC7, 0xC7), (0x00, 0x77, 0xFF), (0x21, 0x55, 0xFF), (0x82, 0x37, 0xFA),
   (0xEB, 0x2F, 0xB5), (0xFF, 0x29, 0x50), (0xFF, 0x22, 0x00), (0xD6, 0x32, 0x00), (0xC4, 0x62, 0x00),
   (0x35, 0x80, 0x00), (0x05, 0x8F, 0x00), (0x00, 0x8A, 0x55), (0x00, 0x99, 0xCC), (0x21, 0x21, 0x21),
   (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xFF, 0xFF, 0xFF), (0x0F, 0xD7, 0xFF), (0x69, 0xA2, 0xFF),
   (0xD4, 0x80, 0xFF), (0xFF, 0x45, 0xF3), (0xFF, 0x61, 0x8B), (0xFF, 0x88, 0x33), (0xFF, 0x9C, 0x12),
   (0xFA, 0xBC, 0x20), (0x9F, 0xE3, 0x0E), (0x2B, 0xF0, 0x35), (0x0C, 0xF0, 0xA4), (0x05, 0xFB, 0xFF),
   (0x5E, 0x5E, 0x5E), (0x0D, 0x0D, 0x0D), (0x0D, 0x0D, 0x0D), (0xFF, 0xFF, 0xFF), (0xA6, 0xFC, 0xFF),
   (0xB3, 0xEC, 0xFF), (0xDA, 0xAB, 0xEB), (0xFF, 0xA8, 0xF9), (0xFF, 0xAB, 0xB3), (0xFF, 0xD2, 0xB0),
   (0xFF, 0xEF, 0xA6), (0xFF, 0xF7, 0x9C), (0xD7, 0xE8, 0x95), (0xA6, 0xED, 0xAF), (0xA2, 0xF2, 0xDA),
   (0x99, 0xFF, 0xFC), (0xDD, 0xDD, 0xDD), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11)
];

pub struct Frame {
   pub data: Vec<u8>,
//    pub cpu: Option<CPU>,
}

impl Frame {
   const WIDTH: usize = 256;
   const HIGHT: usize = 240;

   pub fn new() -> Self {
       Frame {
           data: vec![0; (Frame::WIDTH) * (Frame::HIGHT) * 3],
        //    cpu: None,
       }
   }

   pub fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
       let base = y * 3 * Frame::WIDTH + x * 3;
       if base + 2 < self.data.len() {
           self.data[base] = rgb.0;
           self.data[base + 1] = rgb.1;
           self.data[base + 2] = rgb.2;
       }
   }
}

fn bg_pallette(ppu: &PPU, tile_column: usize, tile_row : usize) -> [u8;4] {
   let attr_table_idx = tile_row / 4 * 8 +  tile_column / 4;
   let attr_byte = ppu.vram[0x3c0 + attr_table_idx];  // note: still using hardcoded first nametable

   let pallet_idx = match (tile_column %4 / 2, tile_row % 4 / 2) {
       (0,0) => attr_byte & 0b11,
       (1,0) => (attr_byte >> 2) & 0b11,
       (0,1) => (attr_byte >> 4) & 0b11,
       (1,1) => (attr_byte >> 6) & 0b11,
       (_,_) => panic!("should not happen"),
   };

   let pallete_start: usize = 1 + (pallet_idx as usize)*4;
   [ppu.palette_table[0], ppu.palette_table[pallete_start], ppu.palette_table[pallete_start+1], ppu.palette_table[pallete_start+2]]
}


pub fn render_all_tiles(ppu: &PPU, chr_rom: &[u8], frame: &mut Frame) {
   let mut tileIdx = 0;
   let mut tile_y = 0;
   let mut tile_x = 0;
//    let bank = get_register_flag(&ppu.ppu_ctrl, ControlFlag::BackgroundPatternTable as u8) as usize;
    let bank = if get_register_flag(
        &ppu.ppu_ctrl,
        ControlFlag::BackgroundPatternTable as u8,
    ) {
        0x1000
    } else {
        0x0000
    };

    for i in 0..0x3c0 {
        let tile = ppu.vram[i] as u16;
       let tile_column = i % 32;
       let tile_row = i / 32;
       let tile = &ppu.chr_rom[(bank + tile as usize * 16)..=(bank + tile as usize * 16 + 15)];
       let palette = bg_pallette(ppu, tile_column, tile_row);

        for y in 0..=7 {
           let mut upper = tile[y];
           let mut lower = tile[y + 8];

           for x in (0..=7).rev() {
               let value = (1 & lower) << 1 | (1 & upper);
               upper = upper >> 1;
               lower = lower >> 1;
               let rgb = match value {
                   0 => SYSTEM_PALLETE[ppu.palette_table[0] as usize],
                   1 => SYSTEM_PALLETE[palette[1] as usize],
                   2 => SYSTEM_PALLETE[palette[2] as usize],
                   3 => SYSTEM_PALLETE[palette[3] as usize],
                   _ => panic!("can't be"),
               };
               frame.set_pixel(tile_column * 8 + x, tile_row * 8 + y, rgb)
           }
       }

    }

    // for chunk in chr_rom[..8192].chunks(16) {
    //     if tileIdx != 0 && tileIdx % 20 == 0 {
    //         tile_y += 10;
    //         tile_x = 0;
    //     }
    //     for byteIdx in 0..=7 {
    //         let plane0 = chunk[byteIdx];
    //         let plane1 = chunk[byteIdx + 8];

    //         for bitIdx in 0..=7 {
    //             let bit = 7 - bitIdx;
    //             let low = (plane0 >> bit) & 1;
    //             let high = (plane1 >> bit) & 1;
    //             let value = low | (high << 1);

    //             let rgb = match value {
    //                 0 => SYSTEM_PALLETE[0x01],
    //                 1 => SYSTEM_PALLETE[0x23],
    //                 2 => SYSTEM_PALLETE[0x27],
    //                 3 => SYSTEM_PALLETE[0x30],
    //                 _ => panic!("can't be"),
    //             };
    //             frame.set_pixel(bitIdx + tile_x, byteIdx + tile_y, rgb)
    //         }


    //     }
    //     tileIdx += 1;
    //     tile_x += 10;
    // }
}

// old version
// pub fn render_all_tiles(ppu: &PPU, chr_rom: &[u8], frame: &mut Frame) {
//    let mut tileIdx = 0;
//    let mut tile_y = 0;
//    let mut tile_x = 0;

//     for chunk in chr_rom[..8192].chunks(16) {
//         if tileIdx != 0 && tileIdx % 20 == 0 {
//             tile_y += 10;
//             tile_x = 0;
//         }
//         for byteIdx in 0..=7 {
//             let plane0 = chunk[byteIdx];
//             let plane1 = chunk[byteIdx + 8];

//             for bitIdx in 0..=7 {
//                 let bit = 7 - bitIdx;
//                 let low = (plane0 >> bit) & 1;
//                 let high = (plane1 >> bit) & 1;
//                 let value = low | (high << 1);

//                 let rgb = match value {
//                     0 => SYSTEM_PALLETE[0x01],
//                     1 => SYSTEM_PALLETE[0x23],
//                     2 => SYSTEM_PALLETE[0x27],
//                     3 => SYSTEM_PALLETE[0x30],
//                     _ => panic!("can't be"),
//                 };
//                 frame.set_pixel(bitIdx + tile_x, byteIdx + tile_y, rgb)
//             }


//         }
//         tileIdx += 1;
//         tile_x += 10;
//     }
// }



pub fn show_tile(chr_rom: &[u8], bank: usize, tile_n: usize) -> Frame {
   let mut frame = Frame::new();
  
   
   print!("Length: {}\n", chr_rom.len());
    // let tile = &chr_rom[(bank * 0x1000 + tile_n * 16)..=(bank * 0x1000 + tile_n * 16 + 15)];
    
    for i in 0..8192 {
        if i % 16 == 0{
            print!("\n");    
        }
        print!("{:02X} ", chr_rom[i]);
    }

    frame
}

pub fn show_tile2(chr_rom: &[u8], bank: usize, tile_n: usize) -> Frame {
   assert!(bank <= 1);

   let mut frame = Frame::new();
   let mut tile_y = 0;
   let mut tile_x = 0;
   let bank = bank * 0x1000;

    for tile_n in 0..255 {
        if tile_n != 0 && tile_n % 20 == 0 {
            tile_y += 10;
            tile_x = 0;
        }
        let tile = &chr_rom[(bank + tile_n * 16)..=(bank + tile_n * 16 + 15)];

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (1 & upper) << 1 | (1 & lower);
                upper >>= 1;
                lower >>= 1;
                let rgb = match value {
                    0 => SYSTEM_PALLETE[0x01],
                    1 => SYSTEM_PALLETE[0x23],
                    2 => SYSTEM_PALLETE[0x27],
                    3 => SYSTEM_PALLETE[0x30],
                    _ => panic!("can't be"),
                };
                frame.set_pixel(tile_x + x, tile_y + y, rgb)
            }
        }
        tile_x += 10;
    }

   frame
}
