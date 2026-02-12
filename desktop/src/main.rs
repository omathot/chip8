use chip8_core::*;
use sdl3::event::Event;
use sdl3::pixels::Color;
use sdl3::rect::Rect;
use sdl3::render::Canvas;
use sdl3::video::Window;
use std::env;
use std::fs;
use std::io::Read;

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

fn main() {
	let args: Vec<_> = env::args().collect();
	if args.len() != 2 {
		eprintln!("Usage: cargo run path/to/game");
		return;
	}

	let sdl_context = sdl3::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
	sdl3::hint::set(sdl3::hint::names::RENDER_VSYNC, "1");
	let window = video_subsystem
		.window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
		.position_centered()
		.opengl()
		.build()
		.unwrap();
	let mut canvas = window.into_canvas();
	canvas.clear();
	canvas.present();

	let mut event_pump = sdl_context.event_pump().unwrap();

	let mut chip8 = Emu::new();
	let mut rom = fs::File::open(&args[1]).expect("Should have been able to open the ROM");
	let mut buff = Vec::new();
	rom.read_to_end(&mut buff).unwrap();
	chip8.load(&buff);

	'gameloop: loop {
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit { .. } => break 'gameloop,
				_ => (),
			}
		}
		chip8.tick();
		draw_screen(&chip8, &mut canvas);
	}
}

fn draw_screen(emu: &Emu, canvas: &mut Canvas<Window>) {
	canvas.set_draw_color(Color::RGB(0, 0, 0));
	canvas.clear();

	let screen_buff = emu.get_display();
	canvas.set_draw_color(Color::RGB(255, 255, 255));
	for (i, pixel) in screen_buff.iter().enumerate() {
		if *pixel {
			let x = (i % SCREEN_WIDTH) as u32;
			let y = (i / SCREEN_WIDTH) as u32;
			let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
			canvas.fill_rect(rect).unwrap();
		}
	}
	canvas.present();
}
