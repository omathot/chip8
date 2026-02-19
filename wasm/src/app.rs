use chip8_core::*;
use leptos::ev;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_use::{use_event_listener, use_raf_fn};
#[cfg(feature = "hydrate")]
use {
	js_sys::Uint8Array,
	std::cell::RefCell,
	std::rc::Rc,
	wasm_bindgen::JsCast,
	wasm_bindgen::prelude::*,
	web_sys::{
		CanvasRenderingContext2d, FileReader, HtmlCanvasElement, HtmlInputElement, KeyboardEvent,
	},
};

const SCALE: usize = 15;
const TICK_PER_FRAME: usize = 5;
const CANVAS_WIDTH: usize = SCREEN_WIDTH * SCALE;
const CANVAS_HEIGHT: usize = SCREEN_HEIGHT * SCALE;

#[component]
pub fn App() -> impl IntoView {
	provide_meta_context();

	view! {
		<Stylesheet id="leptos" href="/pkg/chip8-web.css"/>
		<Link rel="icon" type_="image/png" href="/favicon.png"/>
		<Title text="Chip-8 Emulator"/>
		<main>
			<h1>"Chip-8 Emulator"</h1>
			<Emulator/>
			<h1>"Counter"</h1>
			<Counter/>
		</main>
	}
}

#[cfg(feature = "hydrate")]
thread_local! {
	static ROM_DATA: RefCell<Option<Vec<u8>>> = RefCell::new(None);
}

#[component]
fn Emulator() -> impl IntoView {
	let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
	let (rom_loaded, set_rom_loaded) = signal(false);

	// file upload handler, client-side only
	let on_file_change = {
		#[cfg(feature = "hydrate")]
		{
			move |ev: leptos::ev::Event| {
				let input: HtmlInputElement = event_target(&ev);
				let files = input.files().unwrap();
				if let Some(file) = files.get(0) {
					let reader = FileReader::new().unwrap();
					let reader_clone = reader.clone();

					let onload = Closure::once(Box::new(move |_: web_sys::ProgressEvent| {
						let array_buff = reader_clone.result().unwrap();
						let uint8_array = Uint8Array::new(&array_buff);
						let rom = uint8_array.to_vec();

						ROM_DATA.with(|d| *d.borrow_mut() = Some(rom));
						set_rom_loaded.set(true);
					}));

					reader.set_onload(Some(onload.as_ref().unchecked_ref()));
					onload.forget();
					reader.read_as_array_buffer(&file).unwrap();
				}
			}
		}
		#[cfg(not(feature = "hydrate"))]
		{
			move |_: leptos::ev::Event| {}
		}
	};

	// game loop - runs on client after hydration
	#[cfg(feature = "hydrate")]
	{
		let canvas_node = canvas_ref;
		Effect::new(move |_| {
			if rom_loaded.get() {
				let canvas: HtmlCanvasElement = canvas_node.get().unwrap().into();
				let ctx = canvas
					.get_context("2d")
					.unwrap()
					.unwrap()
					.dyn_into::<CanvasRenderingContext2d>()
					.unwrap();

				let rom = ROM_DATA.with(|d| d.borrow_mut().take().unwrap());
				let mut emu = Emu::new();
				emu.load(&rom);

				start_game(Rc::new(RefCell::new(emu)), ctx, canvas.clone());
			}
		});
	}
	view! {
		<label for="fileinput">"Upload a Chip-8 game: "</label>
		<input type="file" id="fileinput" on:change=on_file_change/>
		<br/>
		<canvas
			node_ref=canvas_ref
			width=CANVAS_WIDTH.to_string()
			height=CANVAS_HEIGHT.to_string()
			tabindex="0"
			style="background: black;"
		/>
	}
}

#[cfg(feature = "hydrate")]
fn start_game(emu: Rc<RefCell<Emu>>, ctx: CanvasRenderingContext2d, canvas: HtmlCanvasElement) {
	let emu_down = emu.clone(); // RC, doesn't clone, increments ref count
	let emu_up = emu.clone();

	// don't care to keep a handle to these
	let _ = use_event_listener(document(), ev::keydown, move |ev: KeyboardEvent| {
		if let Some(k) = key_to_btn(&ev.key()) {
			emu_down.borrow_mut().keypress(k, true);
		}
	});
	let _ = use_event_listener(document(), ev::keyup, move |ev: KeyboardEvent| {
		if let Some(k) = key_to_btn(&ev.key()) {
			emu_up.borrow_mut().keypress(k, false);
		}
	});

	// animation loop
	let pausable = use_raf_fn(move |_args| {
		let mut e = emu.borrow_mut();
		for _ in 0..TICK_PER_FRAME {
			e.tick();
		}
		e.tick_timers();

		ctx.set_fill_style_str("black");
		ctx.fill_rect(0., 0., CANVAS_WIDTH as f64, CANVAS_HEIGHT as f64);
		ctx.set_fill_style_str("white");
		let display = e.get_display();
		for i in 0..(SCREEN_WIDTH * SCREEN_HEIGHT) {
			if display[i] {
				let x = i % SCREEN_WIDTH;
				let y = i / SCREEN_WIDTH;
				ctx.fill_rect(
					(x * SCALE) as f64, // scale back to canvas coords here
					(y * SCALE) as f64,
					SCALE as f64,
					SCALE as f64,
				);
			}
		}
	});
	(pausable.resume)();
}

fn key_to_btn(key: &str) -> Option<usize> {
	match key {
		"1" => Some(0x1),
		"2" => Some(0x2),
		"3" => Some(0x3),
		"4" => Some(0xC),
		"q" => Some(0x4),
		"w" => Some(0x5),
		"e" => Some(0x6),
		"r" => Some(0xD),
		"a" => Some(0x7),
		"s" => Some(0x8),
		"d" => Some(0x9),
		"f" => Some(0xE),
		"z" => Some(0xA),
		"x" => Some(0x0),
		"c" => Some(0xB),
		"v" => Some(0xF),
		_ => None,
	}
}

#[component]
fn Counter() -> impl IntoView {
	let (count, set_count) = signal(0);

	view! {
		<button
			on:click=move |_| {
				*set_count.write() += 1;
			}
			class:red=move || count.get() % 2 == 1
		>
			"Click me: "
			{count}
		</button>
	}
}
