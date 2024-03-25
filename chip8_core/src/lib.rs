use rand::random;
// chip-8 uses a 64x32 monochromatic display
// public for allowing access to the frontend
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const FONTSET_SIZE: usize = 80;
// divided into 5 groups of bytes,
// each byte represents a row in the display as pixels
// for example 0xF0 = 11110000, 0x90 = 10010000
// set each byte in top of each other would display
// 11110000
// 10010000
// 10010000
// 10010000
// 11110000
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

const RAM_SIZE: usize = 4096; // max memory allowed in chip-8
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200; // rams starts reading at address 0x200

pub struct Emu {
    pc: u16, // program counter, keeps tracks which instruction it currently is executing; increments as the game runs
    ram: [u8; RAM_SIZE], // ram for our emulator
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], // since its a monochromatic display, we can store each pixels' color with a boolean
    v_reg: [u8; NUM_REGS], // registers that chip-8 uses, going from V0 to VF (16 in total), VF doubles as the flag register
    i_reg: u16, // for indexing the ram
    sp: u16, // stack pointer, keeps track of the top of the stack
    stack: [u16; STACK_SIZE], // stack for the cpu to read/write to
    keys: [bool; NUM_KEYS],
    dt: u8, // delay timer, typical timer, performs action if it hits 0
    st: u8, // sound timer, emits sound when it hits 0
}

impl Emu {
    pub fn new() -> Self {
        // initalizes all values and arrays to zero (except the program counter)
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };
        // we will use the ram before the start address (0x200) for our sprites as this would be unused in our emulator
        // (better efficiency)
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }
    // resets emulator without needing to create a new object
    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st =  0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }
    // basic push and pop functions for our stack
    fn push(&mut self, val: u16) { 
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }
    // of course, returns the popped value
    fn pop(&mut self) ->  u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
    /* basic loop in our emulator (from the book):
        1. Fetch the value from our game (loaded into RAM) at the memory address stored in our Program Counter.
        2. Decode this instruction.
        3. Execute, which will possibly involve modifying our CPU registers or RAM.
        4. Move the PC to the next instruction and repeat
    */
    pub fn tick(&mut self) {
        // fetch
        let op = self.fetch();
        // decode
        // execute
        self.execute(op);
    }
    // decodes the given opcode and executes it
    fn execute(&mut self, op: u16) {
        // seperates the op code into four hex digits
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // NOP instruction
            // moves to the next opcode (needed for timing or aligment purposes)
            (0, 0, 0, 0) => return,
            // Clear screen instruction
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            // Return from subroutine
            // gets the last address pushed in the stack to continue from after a subroutine
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            },
            // Jump
            // simply moves the pointer counter to the given address
            // opcode beings with 0x1, and the next three digits (nnn) are any three digits
            (1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            },
            // Call subroutine
            // opposite function of Return from subroutine
            // add the current pointer counter to the stack and jump to the given address (nnn)
            (2, _, _, _) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            },
            // Skip next if VX == NN
            // basically acts as an if-else block
            // second digit (x) tells us the V register to use
            // last two digits (nn) tells us the raw value to compare
            // if true go to one instruction
            // if false go somewhere else
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                // uses one of the V registers
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            },
            // Skip next if VX != NN
            // same as previous, but compare if the values are not equal
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                // uses one of the V registers
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            },
            // Skip next if VX == VY
            // similar operations to previous ones
            // however, we use the third digit acts like the second digit
            // as indexer into another V register
            // doesn't use digit4 as this requires it to be 0x0
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            },
            // VX = NN
            // sets the V register equal the second digit
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            }
            // VX += NN
            // adds the value given to the VX register
            // rust will panic in the event of an overflow
            // so we use a different method than the typical + operator
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            },
            // VX = VY
            // like the VX = NN operation, but the source value is from the VY register
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            },
            // VX |= VY
            // applies OR operator to VX register and VY register
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            },
            // VX &= VY
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            },
            // VX ^= VY
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            },
            // VX += VY
            // VX register becomes VX plus VY 
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                // overflowing_add retuns a tuple, which contains the wrapped sum, and a boolean that tells us if an overflow occured               
                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                // VF needs to be set to 1 if an overflow happened in the sum, 0 in case it didn't
                let new_vf = if carry { 1 } else { 0 };
                // sets the new values of VF and VX
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // VX -= VY 
            // same operation as previous but with a substraction
            // VF will work in an opposite way, if an underflow were to happend, VF is set to 0
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);

                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // VX >>= 1
            // performs one right shift on the value in VX
            (8, _, _, 6) => {
                let x = digit2 as usize;
                // gets the dropped off bit and will store it in VF
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            },
            // VX = VY - VX
            // wors the same way as VX -= VY, but with operands in the opposite direction
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);

                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // VX <<= 1
            // similar to the right shift operation, but that overflowed value is stored in VF
            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            },
            // Skip if VX != VY
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            },
            // I = NNN
            // utilizes the I register
            // simply we are setting it as 0xNNN
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            },
            // Jump to V0 + NNN
            // utilizes the first V register (V0)
            // moves the pointer counter to the sum of the value stored in V0 and the raw value 0xNNN
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            },
            // VX = rand() & NN
            // chip8 rng operation
            // random number is AND with two values in the opcode (NN)
            // sets that to the VX register
            // will be using rust's rand library
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                // setting the random variable as u8 is necessary for random() to know which type it is supposed to generate
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            },
            // Draw sprite
            // most complicated opcode
            // chip8 works by drawing sprites, images stored in memory
            // second and third digits give us the V registers we are to fetch our (x,y) coordinates from
            // sprites are always 8 pixels wide
            // can be any number from 1 to 16 pixels tall (specified in the last digit of the opcode)
            // I registers are used to store an address in memory
            // sprites are stored row by row beginning at the addres stored in I
            // if we are told to draw a 3px tall sprite,
            // first row's data is stored at *I, then *I+1, finally *I+2
            // any pixel flipped from white to black or viceverse, VF is set and cleared otherwise
            (0xD, _, _, _) => {
                // getting coordinates (x,y) from the V register for our sprite
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;
                // the last digit determines the sprite's height
                let num_rows = digit4;
                // keep track if any pixels were flipped
                let mut flipped = false;
                // iteration over each row of our sprite
                for y_line in 0..num_rows {
                    // get memory address from where the row's data is stored
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    // iterate over each column in the row
                    for x_line in 0..8 {
                        // use mask to fetch current pixel's bit
                        // only flips if 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // sprites shoudl wrap around screen, so apply modulo
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;
                            // get our pixel's index for our screen array
                            let idx = x + SCREEN_WIDTH * y;
                            // check if we're about to flip the pixel and set
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                // set the VF register
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            },
            // Skip if key pressed
            // user input
            // 16 possible keys, 0 to 0xF
            // this checks if the index stored in VX is pressed
            // if so, skip to the next instruction
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            },
            // Skip if key not pressed
            // same as previous one but as an inequality as to whether the key was pressed
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            },
            // VX = DT
            // stores the delay timer in VX register
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            },
            // Wait for key press
            // instruction is blocking
            // whole game pauses until the player presses a key
            // loops endlessly unitl something in the keys array turns true
            (0xF, _, 0, 0xa) => {
                let x = digit2 as usize;
                let mut pressed = false;
                // cycles through the keys array
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                // redo opcode if key wasn't pressed
                if !pressed {
                    self.pc -= 2
                }
            },
            // DT = VX
            // overwrites the delay timer to whatever the value in VX register is
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            },
            // ST = VX 
            // same as previous one, but overwrites instead the sound timer
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            },
            // I += VX
            // increments the I register to the value in VX
            // roll back to 0 in case of an overflow, so we are wrapping_add
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            },
            // Set I to Font Address
            // using the first 512 values in the ram (was set as the font data)
            // take the number received in the opcode (x), and store the ram address ..
            // .. of that sprite into the I register
            // because we start at the beginning of the ram 0x0, and each character is ..
            // .. 5 bytes, we can simply multiply the value times 5
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            },
            // I = BCD of VX
            // stores the binary-coded decimal of number stored in VX into the I register
            // BCD converts a hexadecimal number back into a pseudo-decimal number to print to the user
            // e.g. points, high scores, etc.
            // for example, 0x64 (or 100 in decimnal), gives us 0x1, 0x0, 0x0, which prints out 100
            // since it gives us bytes, we will store the BCD into the ram, beginning in the address in I register
            // since VX stores 8-bit numbers (0..255), we are always going to end up with three bytes
            // / slow solution as a trade-off for readibility /
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                // converted as float so that we can use division and modulo to get decimal digits
                let vx = self.v_reg[x] as f32;
                // fetch hunderds by dividing by 100 and tossing the decimals
                let hundreds = (vx / 100.0).floor() as u8;
                // fetch tenths by dividing by 10 and tossing the ones and the decimal
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                // fetch ones by tossing the hunders and the tens
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            },
            // Store V0 - VX into I
            // final two instructions populate the V register V0 through VX (incl.) ..
            // .. with the same range of values from ram, begins with the address in the I register
            // stores the values into ram
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            }
            // Load I into V0 - VX
            // opposite of previous instruction
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx];
                }
            },
            // match case for everything else
            // would probably never reach here, but rust demands it
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }
    // gets the opcode and returns it, each opcode are 2 bytes
    fn fetch(&mut self) -> u16 {
        // remember, pc is the index of the current instruction we are executing
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        // combines both bytes into 2 bytes as big endian
        let op = (higher_byte << 8) | lower_byte;
        // increments pointer counter two bytes for the next operation
        self.pc += 2;
        op
    }
    // each cpu cycle, each timer (delay and sound) will decrease once every frame
    pub fn tick_timers(&mut self) {
        // each timer will decrease unless they are 0
        // in these case they will remain unchanged until changed by the game manually
        if self.dt > 0 {
            self.dt -= 1;
        }
        // when sound timer reaches 0 it will emit a beep
        if self.st > 0 {
            if self.st == 1 {
                // emit a beep
            }
            self.st -= 1;
        }
    }
    // public function that gives a pointer to the display for the frontend
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }
    // handles user key presses
    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }
    // copy data into ram
    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }
}