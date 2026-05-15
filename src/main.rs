pub mod cpu6502;
pub mod instructions;
pub mod rom;
pub mod bus;
pub mod ppu;
pub mod graphics;

use crate::cpu6502::trace;
use crate::cpu6502::{CPU};
use crate::cpu6502::new_cpu;
use crate::rom::Rom;
use crate::bus::Bus;
use crate::graphics::frame::show_tile;
use crate::graphics::frame::render_all_tiles;
use crate::graphics::frame::Frame;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn main() {
    let rom_data = std::fs::read("./rom/pacman.nes").expect("Failed to read ROM file");
    let rom = Rom::parse_nes_rom(rom_data).expect("Failed to parse ROM");
    rom.check_validity().expect("ROM validity check failed");

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("NES Emulator", 256, 240)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    // SAFETY: texture_creator outlives all textures created from it (both live until process exit).
    let mut texture: sdl2::render::Texture<'static> = unsafe {
        std::mem::transmute(
            texture_creator
                .create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGB24, 256, 240)
                .unwrap(),
        )
    };

    let mut event_pump = sdl_context.event_pump().unwrap();

    let chr_rom = rom.chr_rom.clone().unwrap();
    let mut bus = Bus::new(rom);
    let mut frame = Frame::new();

    bus.set_nmi_callback(move |ppu: &ppu::PPU| {
        render_all_tiles(ppu, &chr_rom, &mut frame);

        texture.update(None, &frame.data, 256 * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        println!("NMI triggered");
    });
    let mut cpu: CPU = new_cpu(bus);
    cpu.reset();
    cpu.program_counter = 0xC000;
    cpu.run_with_callback(move |cpu| {
        // println!("{}", trace(cpu));
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                _ => {}
            }
        }
    });
}
