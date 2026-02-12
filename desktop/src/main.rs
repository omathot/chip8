use chip8_core::*;
use sdl3::event::Event;
use std::env;

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

	'gameloop: loop {
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit { .. } => break 'gameloop,
				_ => (),
			}
		}
	}
}
