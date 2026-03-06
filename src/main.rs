use rand::Rng;
use minifb::{Key, Window, WindowOptions};

struct Chip8 {
    memory: [u8; 4096],
    display: [bool;2048],
    v: [u8; 16],
    sp: u8,
    pc: u16,
    i: u16,
    stack: [u16; 16],
    delay_timer: u8,
    sound_timer: u8,
    keys: [bool; 16],
}
impl Chip8 {
    fn new() -> Self {
    let mut chip8 = Chip8 {
        memory: [0; 4096],
        display: [false; 2048],
        v: [0; 16],
        sp: 0,
        pc: 0x200,
        i: 0,
        sound_timer: 0,
        delay_timer: 0,
        stack: [0; 16],
        keys: [false; 16],
    };  // semicolon here is fine because we assigned it to chip8

    let fonts: [u8; 80] = [
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

    for i in 0..80 {
        chip8.memory[i] = fonts[i];
    }

    chip8  // return — no semicolon!
}

fn load_rom(&mut self, data: &[u8]) {
        // copy ROM bytes into memory starting at 0x200
        // hint: loop over data, write each byte
        // memory[0x200 + i] = data[i]
        for i in 0..data.len() {
            self.memory[0x200 + i] = data[i];
        }
    }
fn get_framebuffer(&self) -> Vec<u32> {
    self.display.iter().map(|&pixel| {
        if pixel { 0x00E0F8D0 } else { 0x00081820 }
    }).collect()
}

fn step(&mut self) {
    let byte1: u8 = self.memory[self.pc as usize];
    let byte2: u8 = self.memory[self.pc as usize + 1];
    let opcode: u16 = (byte1 as u16) << 8 | (byte2 as u16);
    self.pc += 2;
    println!("opcode: {:#06X}", opcode); 

    let nibble = (opcode & 0xF000) >> 12;
    let nnn    =  opcode & 0x0FFF;
    let x  = ((opcode & 0x0F00) >> 8) as usize;
    let y  = ((opcode & 0x00F0) >> 4) as usize;
    let kk =  (opcode & 0x00FF) as u8;

    match nibble {
        0x0 => {
            match opcode {
                0x00E0 => {
                    self.display = [false; 2048];
                }
                0x00EE => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                }
                _ => println!("unknown 0x0 opcode: {:#06X}", opcode),
            }
        }
        0x1 => {
            // JP NNN — jump to address NNN
            // what do you think goes here?
            self.pc = nnn;
        }
        0x2 => {
            // push current PC to stack, jump to NNN
            self.stack[self.sp as usize] = self.pc;
            self.sp += 1;
            self.pc = nnn;
        }
        0x3 =>{
            // SE Vx, KK — skip next instruction if Vx = KK
            if self.v[x] == kk {
                self.pc += 2;
            }
        }
        0x4 =>{
            // SNE Vx, KK — skip next instruction if Vx != KK
            if self.v[x] != kk {
            self.pc += 2;
            }
        }
        0x5 => {
            // SE Vx, Vy — skip next instruction if Vx = Vy
            if self.v[x] == self.v[y] {
                self.pc += 2;
            }
        }
        0x6 => {
            // LD Vx, KK — set Vx = KK
            self.v[x] = kk;
        }
        0x7 =>{
            // ADD Vx, KK — add KK to Vx
            self.v[x] = self.v[x].wrapping_add(kk);
        }

        0x8 =>{
            match opcode & 0x000F {
                0x0 => self.v[x] = self.v[y], // LD Vx, Vy
                0x1 => self.v[x] |= self.v[y], // OR Vx, Vy
                0x2 => self.v[x] &= self.v[y], // AND Vx, Vy
                0x3 => self.v[x] ^= self.v[y], // XOR Vx, Vy
                0x4 => { // ADD Vx, Vy with carry
                    let (sum, carry) = self.v[x].overflowing_add(self.v[y]);
                    self.v[x] = sum;
                    self.v[0xF] = if carry { 1 } else { 0 };
                }
                0x5 => { // SUB Vx, Vy with borrow
                    let (diff, borrow) = self.v[x].overflowing_sub(self.v[y]);
                    self.v[x] = diff;
                    self.v[0xF] = if borrow { 0 } else { 1 };
                }
                0x6 => { // SHR Vx {, Vy}
                    self.v[0xF] = self.v[x] & 0x1; // store least significant bit before shift
                    self.v[x] >>= 1;
                }
                0x7 => { // SUBN Vx, Vy with borrow
                    let (diff, borrow) = self.v[y].overflowing_sub(self.v[x]);
                    self.v[x] = diff;
                    self.v[0xF] = if borrow { 0 } else { 1 };
                }
                0xE => { // SHL Vx {, Vy}
                    self.v[0xF] = (self.v[x] & 0x80) >> 7; // store most significant bit before shift
                    self.v[x] <<= 1;
                }
                _ => println!("unknown opcode: {:#06X}", opcode),
            }
        }

        0x9 => {
            // SNE Vx, Vy — skip next instruction if Vx != Vy
            if self.v[x] != self.v[y] {
                self.pc += 2;
            }
        }

        0xA =>{
            //d
            self.i = nnn;
        }

        0xB => {
            // JP V0, NNN — jump to address NNN + V0
            self.pc = nnn + self.v[0] as u16;
        }

        0xC => {
            let random_byte: u8 = rand::random::<u8>();
            self.v[x] = random_byte & kk;
        }
        0xD =>{
                let height = (opcode & 0x000F) as usize;
                let x_coord = self.v[x] as usize;
                let y_coord = self.v[((opcode & 0x00F0) >> 4) as usize] as usize;
                self.v[0xF] = 0; // reset collision flag
                for row in 0..height {
                    let sprite_byte = self.memory[(self.i + row as u16) as usize];
                    for col in 0..8 {
                        if (sprite_byte & (0x80 >> col)) != 0 {
                            let display_index = ((y_coord + row) % 32) * 64 + ((x_coord + col) % 64);
                            if self.display[display_index] {
                                self.v[0xF] = 1; // set collision flag
                            }
                            self.display[display_index] ^= true; // XOR pixel
                        }
                    }
                }
        }

        0xE => {
            match opcode & 0x00FF {
                0x9E => {
                    // SKP Vx — skip next instruction if key with the value of Vx is pressed
                     if self.keys[self.v[x] as usize] {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    // SKNP Vx — skip next instruction if key with the value of Vx is not pressed
                    // for now, we can just pretend no keys are pressed, so always skip
                    if !self.keys[self.v[x] as usize] {
                        self.pc += 2;
                    }
                }
                _ => println!("unknown opcode: {:#06X}", opcode),
            }
        }

        0xF => {
    match opcode & 0x00FF {
        0x07 => { self.v[x] = self.delay_timer; }
        0x15 => { self.delay_timer = self.v[x]; }
        0x18 => { self.sound_timer = self.v[x]; }
        0x1E => { self.i += self.v[x] as u16; }
        0x29 => {
            self.i = self.v[x] as u16 * 5;
        }  // font — we'll do this after fonts loaded
        0x33 => { 
            let value = self.v[x];
            let hundred = value/100;
            let ten = (self.v[x] / 10) % 10;
            let one = value % 10;
            self.memory[self.i as usize] = hundred;
            self.memory[self.i as usize + 1] = ten;
            self.memory[self.i as usize + 2] = one;
        }  // BCD — look this up, it's interesting
        0x55 => { 
            for i in 0..=x {
                self.memory[self.i as usize + i] = self.v[i];
            }
        }  // store registers
        0x65 => { 
            for i in 0..=x {
                self.v[i] = self.memory[self.i as usize + i];
            }
         }  // load registers
        _ => println!("unknown 0xF opcode: {:#06X}", opcode),
    }
}
        
        _ => println!("unknown opcode: {:#06X}", opcode),
    }
}
}
fn main() {
    let mut chip8 = Chip8::new();
    let rom = std::fs::read("roms/SpaceInvaders.ch8").expect("could not read rom");
    chip8.load_rom(&rom);

    let mut window = Window::new(
        "Chip-8",
        640,
        320,
        WindowOptions::default(),
    ).unwrap();

    window.set_target_fps(60);  // set BEFORE the loop

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // run steps
        for _ in 0..10 {
            chip8.step();
        }

        // update display
        let buffer = chip8.get_framebuffer();
        chip8.keys[0]  = window.is_key_down(Key::X);
        chip8.keys[1]  = window.is_key_down(Key::Key1);
        chip8.keys[2]  = window.is_key_down(Key::Key2);
        chip8.keys[3]  = window.is_key_down(Key::Key3);
        chip8.keys[4]  = window.is_key_down(Key::Q);
        chip8.keys[5]  = window.is_key_down(Key::W);
        chip8.keys[6]  = window.is_key_down(Key::E);
        chip8.keys[7]  = window.is_key_down(Key::A);
        chip8.keys[8]  = window.is_key_down(Key::S);
        chip8.keys[9]  = window.is_key_down(Key::D);
        chip8.keys[10] = window.is_key_down(Key::Z);
        chip8.keys[11] = window.is_key_down(Key::C);
        chip8.keys[12] = window.is_key_down(Key::Key4);
        chip8.keys[13] = window.is_key_down(Key::R);
        chip8.keys[14] = window.is_key_down(Key::F);
        chip8.keys[15] = window.is_key_down(Key::V);

        if chip8.delay_timer > 0 {
            chip8.delay_timer -= 1;
        }
        if chip8.sound_timer > 0 {
            chip8.sound_timer -= 1;
        }

        window.update_with_buffer(&buffer, 64, 32).unwrap();
    }
}