use chip8_core::*;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::rect::Rect;
use sdl3::render::Canvas;
use sdl3::video::Window;
use std::env;
use std::fs;
use std::io::Read;

// default SCREEN_WIDTH/HEIGHT are too small for modern screens
const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const TICK_PER_FRAME: usize = 5; // how many opcodes per frame to process

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
				Event::Quit { .. }
				| Event::KeyDown {
					keycode: Some(Keycode::Escape),
					..
				} => break 'gameloop,
				Event::KeyDown {
					keycode: Some(key), ..
				} => {
					if let Some(k) = key_to_btn(key) {
						chip8.keypress(k, true);
					}
				}
				Event::KeyUp {
					keycode: Some(key), ..
				} => {
					if let Some(k) = key_to_btn(key) {
						chip8.keypress(k, false);
					}
				}
				_ => (),
			}
		}
		for _ in 0..TICK_PER_FRAME {
			chip8.tick();
		}
		chip8.tick_timers();
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

/*
	Keyboard -> Chip8
	1234		123C
	QWER		456D
	ASDF		789E
	ZXCV		A0BF
*/
fn key_to_btn(key: Keycode) -> Option<usize> {
	match key {
		Keycode::_1 => Some(0x1),
		Keycode::_2 => Some(0x2),
		Keycode::_3 => Some(0x3),
		Keycode::_4 => Some(0xC),
		Keycode::Q => Some(0x4),
		Keycode::W => Some(0x5),
		Keycode::E => Some(0x6),
		Keycode::R => Some(0xD),
		Keycode::A => Some(0x7),
		Keycode::S => Some(0x8),
		Keycode::D => Some(0x9),
		Keycode::F => Some(0xE),
		Keycode::Z => Some(0xA),
		Keycode::X => Some(0x0),
		Keycode::C => Some(0xB),
		Keycode::V => Some(0xF),
		_ => None,
	}
}
