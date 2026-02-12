use rand::random;

/*
	General relevant info

	-	every sprite in chip8 is 8 pixels wide
	meaning a row of pixels is represented by a u8

	-	v_regs[0xF] (the final register) doubles as the flag register
	example carry flag
*/
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

const START_ADDR: u16 = 0x200;

// each hex is a u8, 5 rows make up the char
const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
	0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
	0x20, 0x60, 0x20, 0x20, 0x70, // 1
	0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
	0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
	0x90, 0x90, 0xF0, 0x10, 0x10, // 4
	0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
	0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
	0xF0, 0x10, 0x20, 0x40, 0x40, // 7
	0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
	0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
	0xF0, 0x90, 0xF0, 0x90, 0x90, // A
	0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
	0xF0, 0x80, 0x80, 0x80, 0xF0, // C
	0xE0, 0x90, 0x90, 0x90, 0xE0, // D
	0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
	0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emu {
	pc: u16,                                      // program counter
	ram: [u8; RAM_SIZE],                          // available ram
	screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], // black or white
	v_regs: [u8; NUM_REGS],                       // 16 8bytes v registers for fast operations
	i_reg: u16,                                   // one 16byte i register to index into ram
	stack: [u16; STACK_SIZE],
	sp: u16,
	keys: [bool; NUM_KEYS],
	dt: u8, // normal timer (delay timer, perform some action when hits 0)
	st: u8, // sound timer (emits when hits 0)
}
impl Emu {
	pub fn new() -> Self {
		let mut new_emu = Self {
			pc: START_ADDR,
			ram: [0; RAM_SIZE],
			screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
			v_regs: [0; NUM_REGS],
			i_reg: 0,
			stack: [0; STACK_SIZE],
			sp: 0,
			keys: [false; NUM_KEYS],
			dt: 0,
			st: 0,
		};
		new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
		new_emu
	}
	pub fn reset(&mut self) {
		self.pc = START_ADDR;
		self.ram = [0; RAM_SIZE];
		self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
		self.v_regs = [0; NUM_REGS];
		self.i_reg = 0;
		self.stack = [0; STACK_SIZE];
		self.sp = 0;
		self.keys = [false; NUM_KEYS];
		self.dt = 0;
		self.st = 0;
		self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
	}
	pub fn tick(&mut self) {
		let op = self.fetch();
		self.execute(op);
	}
	pub fn tick_timers(&mut self) {
		if self.dt > 0 {
			self.dt -= 1;
		}

		if self.st > 0 {
			if self.st == 1 {
				// emit
			}
			self.st -= 1;
		}
	}

	fn fetch(&mut self) -> u16 {
		let higher_byte = self.ram[self.pc as usize] as u16;
		let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
		let op = (higher_byte << 8) | lower_byte;
		self.pc += 2;
		op
	}
	fn execute(&mut self, op: u16) {
		let [high, low] = op.to_be_bytes();
		let nibbles = (high >> 4, high & 0xF, low >> 4, low & 0xF);

		match nibbles {
			// NOP
			(0, 0, 0, 0) => return,
			// CLS
			(0, 0, 0xE, 0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT],
			// RET
			(0, 0, 0xE, 0xE) => {
				let ret_addr = self.pop();
				self.pc = ret_addr;
			}
			// JP ADDR
			(1, _, _, _) => {
				let nnn = op & 0xFFF;
				self.pc = nnn;
			}
			// CALL ADDR
			(2, _, _, _) => {
				let nnn = op & 0xFFF;
				self.push(self.pc);
				self.pc = nnn;
			}
			// Skip if Vx == kk
			(3, _, _, _) => {
				let x = nibbles.1 as usize;
				let kk = (op & 0xFF) as u8;
				if self.v_regs[x] == kk {
					self.pc += 2;
				}
			}
			// Skip if Vx != kk
			(4, _, _, _) => {
				let x = nibbles.1 as usize;
				let kk = (op & 0xFF) as u8;
				if self.v_regs[x] != kk {
					self.pc += 2;
				}
			}
			// Skip if Vx == Vy
			(5, _, _, 0) => {
				let x = nibbles.1 as usize;
				let y = nibbles.2 as usize;
				if self.v_regs[x] == self.v_regs[y] {
					self.pc += 2;
				}
			}
			// Set Vx = kk
			(6, _, _, _) => {
				let x = nibbles.1 as usize;
				let kk = (op & 0xFF) as u8;
				self.v_regs[x] = kk;
			}
			// Set Vx += kk
			(7, _, _, _) => {
				let x = nibbles.1 as usize;
				let kk = (op & 0xFF) as u8;
				self.v_regs[x] = self.v_regs[x].wrapping_add(kk);
			}
			// Set Vx = Vy
			(8, _, _, 0) => {
				let (x, y) = xy_nibbles(nibbles);
				self.v_regs[x] = self.v_regs[y];
			}
			// Set Vx |= Vy
			(8, _, _, 1) => {
				let (x, y) = xy_nibbles(nibbles);
				self.v_regs[x] |= self.v_regs[y];
			}
			// Set Vx &= Vy
			(8, _, _, 2) => {
				let (x, y) = xy_nibbles(nibbles);
				self.v_regs[x] &= self.v_regs[y];
			}
			// Set Vx ^= Vy (XOR)
			(8, _, _, 3) => {
				let (x, y) = xy_nibbles(nibbles);
				self.v_regs[x] ^= self.v_regs[y];
			}
			// Set Vx += Vy, VF = carry
			(8, _, _, 4) => {
				let (x, y) = xy_nibbles(nibbles);
				let (new_vx, carry) = self.v_regs[x].overflowing_add(self.v_regs[y]);
				let new_vf = if carry { 1 } else { 0 };

				self.v_regs[x] = new_vx;
				self.v_regs[0xF] = new_vf;
			}
			// Set Vx -= Vy, VF = NOT BORROW
			(8, _, _, 5) => {
				let (x, y) = xy_nibbles(nibbles);
				let (new_vx, borrow) = self.v_regs[x].overflowing_sub(self.v_regs[y]);
				let new_vf = if borrow { 0 } else { 1 };

				self.v_regs[x] = new_vx;
				self.v_regs[0xF] = new_vf;
			}
			// Set VX >>= 1
			(8, _, _, 6) => {
				let x = nibbles.1 as usize;
				let lsb = self.v_regs[x] & 1;
				self.v_regs[x] >>= 1;
				self.v_regs[0xF] = lsb;
			}
			// Set Vx = Vy - Vx, VF = NOT BORROW
			(8, _, _, 7) => {
				let (x, y) = xy_nibbles(nibbles);
				let (new_vx, borrow) = self.v_regs[y].overflowing_sub(self.v_regs[x]);
				let new_vf = if borrow { 0 } else { 1 };

				self.v_regs[x] = new_vx;
				self.v_regs[0xF] = new_vf;
			}
			// SET Vx <<= 1
			(8, _, _, 8) => {
				let x = nibbles.1 as usize;
				let msb = (self.v_regs[x] >> 7) & 1;
				self.v_regs[x] <<= 1;
				self.v_regs[0xF] = msb;
			}
			// Skip if Vx != Vy
			(9, _, _, 0) => {
				let (x, y) = xy_nibbles(nibbles);
				if self.v_regs[x] != self.v_regs[y] {
					self.pc += 2;
				}
			}
			// Set I = nnn
			(0xA, _, _, _) => {
				let nnn = op & 0xFFF;
				self.i_reg = nnn;
			}
			// Jump to nnn + V0
			(0xB, _, _, _) => {
				let nnn = op & 0xFFF;
				self.pc = nnn + (self.v_regs[0] as u16);
			}
			// Set Vx = rand & kk
			(0xC, _, _, _) => {
				let x = nibbles.1 as usize;
				let kk = (op & 0xFF) as u8;
				let rng: u8 = random();
				self.v_regs[x] = rng & kk;
			}
			// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
			// where n : num of rows for sprite (width (columns) are always 8 pixels)
			(0xD, _, _, _) => {
				let (x, y) = xy_nibbles(nibbles);
				let n = op & 0xF;
				let x_coord = self.v_regs[x] as u16;
				let y_coord = self.v_regs[y] as u16;
				let mut flipped = false;
				for y_line in 0..n {
					let addr = self.i_reg + y_line as u16;
					let pixels = self.ram[addr as usize];
					for x_line in 0..8 {
						if pixels & (0b1000_0000 >> x_line) != 0 {
							let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
							let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

							let idx = x + SCREEN_WIDTH * y;
							flipped |= self.screen[idx];
							self.screen[idx] ^= true;
						}
					}
				}
				if flipped {
					self.v_regs[0xF] = 1;
				} else {
					self.v_regs[0xF] = 0;
				}
			}
			// Skip if key value Vx IS pressed
			(0xE, _, 9, 0xE) => {
				let x = nibbles.1 as usize;
				let vx = self.v_regs[x];
				let key = self.keys[vx as usize];
				if key {
					self.pc += 2;
				}
			}
			// Skip if key value VX IS NOT pressed
			(0xE, _, 0xA, 1) => {
				let x = nibbles.1 as usize;
				let vx = self.v_regs[x];
				let key = self.keys[vx as usize];
				if !key {
					self.pc += 2;
				}
			}
			// Set Vx = DT
			(0xF, _, 0, 7) => {
				let x = nibbles.1 as usize;
				self.v_regs[x] = self.dt;
			}
			// Wait for keypress, store in VX
			(0xF, _, 0, 0xA) => {
				let x = nibbles.1 as usize;
				let mut pressed = false;
				for i in 0..self.keys.len() {
					if self.keys[i] {
						self.v_regs[x] = i as u8;
						pressed = true;
						break;
					}
				}
				if !pressed {
					self.pc -= 2;
				}
			}
			// Set Delay Timer = Vx
			(0xF, _, 1, 5) => {
				let x = nibbles.1 as usize;
				self.dt = self.v_regs[x];
			}
			// Set Sound Timer = Vx
			(0xF, _, 1, 8) => {
				let x = nibbles.1 as usize;
				self.st = self.v_regs[x];
			}
			// Set I += Vx
			(0xF, _, 1, 0xE) => {
				let x = nibbles.1 as usize;
				self.i_reg = self.i_reg.wrapping_add(self.v_regs[x] as u16);
			}
			// Set I = Vx sprite RAM address
			// We put the sprite data at start of RAM, and they're 5 bytes each
			// so simply: their value * 5
			(0xF, _, 2, 9) => {
				let x = nibbles.1 as usize;
				let c = self.v_regs[x] as u16;
				self.i_reg = c * 5;
			}
			// Set ram[i_reg..i_reg + 2] = BCD of Hex
			// translate hex (base 16) back to base 10 to display to players
			// max value 255, always 3 bytes for 3 digits
			// TODO: Optimize
			(0xF, _, 3, 3) => {
				let x = nibbles.1 as usize;
				let vx = self.v_regs[x] as f32;

				let hundreds = (vx / 100.).floor() as u8;
				let tens = ((vx / 10.) % 10.).floor() as u8;
				let ones = (vx % 10.).floor() as u8;

				self.ram[self.i_reg as usize] = hundreds;
				self.ram[(self.i_reg + 1) as usize] = tens;
				self.ram[(self.i_reg + 2) as usize] = ones;
			}
			// Set Memory[i_reg + idx] = V[idx]
			(0xF, _, 5, 5) => {
				let x = nibbles.1 as usize;
				let i = self.i_reg as usize;
				for idx in 0..=x {
					self.ram[i + idx] = self.v_regs[idx];
				}
			}
			(0xF, _, 6, 5) => {
				let x = nibbles.1 as usize;
				let i = self.i_reg as usize;
				for idx in 0..=x {
					self.v_regs[idx] = self.ram[i + idx];
				}
			}
			(_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
		};
	}

	fn push(&mut self, val: u16) {
		self.stack[self.sp as usize] = val;
		self.sp += 1;
	}
	fn pop(&mut self) -> u16 {
		if self.sp == 0 {
			eprintln!("Attempted to pop on empty stack");
			panic!();
		}
		self.sp -= 1;
		self.stack[self.sp as usize]
	}

	// frontend
	pub fn get_display(&self) -> &[bool] {
		&self.screen
	}
	pub fn keypress(&mut self, idx: usize, pressed: bool) {
		self.keys[idx] = pressed;
	}
	pub fn load(&mut self, data: &[u8]) {
		let start = START_ADDR as usize;
		let end = (START_ADDR as usize) + data.len();
		self.ram[start..end].copy_from_slice(data);
	}
}

fn xy_nibbles(nibbles: (u8, u8, u8, u8)) -> (usize, usize) {
	let x = nibbles.1 as usize;
	let y = nibbles.2 as usize;
	(x, y)
}

#[cfg(test)]
mod tests {
	// use super::*;

	#[test]
	fn it_works() {}
}
