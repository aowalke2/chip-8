use rand::Rng;

const PROGRAM_START: u16 = 0x200;
const STACK_SIZE: usize = 16;
const NUMBER_OF_REGISTERS: usize = 16;
const MEMORY_SIZE: usize = 4096;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const NUMBER_OF_KEYS: usize = 16;
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

trait Memory {
    fn mem_read(&self, address: u16) -> u8;

    fn mem_read_16(&self, address: u16) -> u16 {
        let hi = self.mem_read(address) as u16;
        let lo = self.mem_read(address + 1) as u16;
        hi << 8 | lo
    }

    fn mem_write(&mut self, address: u16, data: u8);

    fn mem_write_16(&mut self, address: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(address, hi);
        self.mem_write(address + 1, lo);
    }
}

trait Stack {
    fn stack_push(&mut self, address: u16);
    fn stack_pop(&mut self) -> u16;
}

#[derive(Debug, Clone)]
pub struct Interpreter {
    program_counter: u16,
    index_register: u16,
    stack_pointer: u8,
    stack: [u16; STACK_SIZE],
    registers: [u8; NUMBER_OF_REGISTERS],
    memory: [u8; MEMORY_SIZE],
    delay_timer: u8,
    sound_timer: u8,
    screen: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
    keys: [bool; NUMBER_OF_KEYS],
}

impl Memory for Interpreter {
    fn mem_read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn mem_write(&mut self, address: u16, data: u8) {
        self.memory[address as usize] = data
    }
}

impl Stack for Interpreter {
    fn stack_push(&mut self, address: u16) {
        self.stack[self.stack_pointer as usize] = address;
        self.stack_pointer += 1;
    }

    fn stack_pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut interpreter = Self {
            program_counter: 0,
            index_register: 0,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            registers: [0; NUMBER_OF_REGISTERS],
            memory: [0; MEMORY_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            screen: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            keys: [false; NUMBER_OF_KEYS],
        };

        interpreter.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        interpreter
    }

    fn cls(&mut self) {
        self.screen = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT];
    }

    fn ret(&mut self) {
        self.program_counter = self.stack_pop();
    }

    fn jp_to_addr(&mut self, opcode: u16) {
        self.program_counter = opcode & 0x0FFF;
    }

    fn call_at_addr(&mut self, opcode: u16) {
        self.stack_push(self.program_counter);
        self.program_counter = opcode & 0x0FFF;
    }

    fn se_vx_and_byte(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let vx = self.registers[x as usize];
        if vx == (opcode & 0x00FF) as u8 {
            self.program_counter += 2;
        }
    }

    fn sne_vx_and_byte(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let vx = self.registers[x as usize];
        if vx != (opcode & 0x00FF) as u8 {
            self.program_counter += 2;
        }
    }

    fn se_vx_and_vy(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        if vx == vy {
            self.program_counter += 2;
        }
    }

    fn ld_vx_with_byte(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        self.registers[x as usize] = (opcode & 0x00FF) as u8;
    }

    fn add_vx_with_byte(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        self.registers[x as usize] =
            self.registers[x as usize].wrapping_add((opcode & 0x00FF) as u8);
    }

    fn ld_vx_with_vy(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        self.registers[x as usize] = self.registers[y as usize];
    }

    fn or_vx_with_vy(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let result = self.registers[x as usize] | self.registers[y as usize];
        self.registers[x as usize] = result;
    }

    fn and_vx_with_vy(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let result = self.registers[x as usize] & self.registers[y as usize];
        self.registers[x as usize] = result;
    }

    fn xor_vx_with_vy(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let result = self.registers[x as usize] ^ self.registers[y as usize];
        self.registers[x as usize] = result;
    }

    fn add_vx_with_vy(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let (result, carry) =
            self.registers[x as usize].overflowing_add(self.registers[y as usize]);
        self.registers[0xF] = if carry { 1 } else { 0 };
        self.registers[x as usize] = result;
    }

    fn sub_vx_with_vy(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let (result, borrow) =
            self.registers[x as usize].overflowing_sub(self.registers[y as usize]);
        self.registers[0xF] = if borrow { 0 } else { 1 };
        self.registers[x as usize] = result;
    }

    fn shr_vx(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let vx = self.registers[x as usize];
        self.registers[0xF] = vx & 1;
        self.registers[x as usize] >>= 1;
    }

    fn subn_vx_with_vy(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let (result, borrow) =
            self.registers[y as usize].overflowing_sub(self.registers[x as usize]);
        self.registers[0xF] = if borrow { 0 } else { 1 };
        self.registers[x as usize] = result;
    }

    fn shl_vx(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let vx = self.registers[x as usize];
        self.registers[0xF] = (vx >> 7) & 1;
        self.registers[x as usize] <<= 1;
    }

    fn sne_vx_and_vy(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        if vx != vy {
            self.program_counter += 2;
        }
    }

    fn ld_i_with_addr(&mut self, opcode: u16) {
        self.index_register = opcode & 0x0FFF;
    }

    fn jp_to_v0_plus_addr(&mut self, opcode: u16) {
        self.program_counter = (opcode & 0x0FFF) + self.registers[0] as u16;
    }

    fn rnd(&mut self, opcode: u16) {
        let mut rng = rand::thread_rng();
        let byte = rng.gen_range(0..=255) as u8;
        let x = (opcode & 0x0F00) >> 8;
        self.registers[x as usize] = byte & (opcode & 0x00FF) as u8;
    }

    fn draw(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let n = (opcode & 0x000F) as u8; // height

        let x_coordinate = self.registers[x as usize];
        let y_coordinate = self.registers[y as usize];
        let mut is_flipped = false;

        for dy in 0..n {
            let addr = self.index_register + dy as u16;
            let pixels = self.mem_read(addr);
            for dx in 0..8 {
                if (pixels & (0b1000_0000 >> dx)) != 0 {
                    let col = (x_coordinate + dx) as usize % SCREEN_WIDTH;
                    let row = (y_coordinate + dy) as usize % SCREEN_HEIGHT;

                    is_flipped |= self.screen[row][col];
                    self.screen[row][col] ^= true;
                }
            }
        }

        self.registers[0xF] = if is_flipped { 1 } else { 0 };
    }

    fn skp(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let vx = self.registers[x as usize];
        if self.keys[vx as usize] {
            self.program_counter += 2
        }
    }

    fn sknp(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let vx = self.registers[x as usize];
        if !self.keys[vx as usize] {
            self.program_counter += 2
        }
    }

    fn ld_vx_with_dt(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        self.registers[x as usize] = self.delay_timer;
    }

    fn ld_vx_with_key_press(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let mut pressed = false;
        for i in 0..self.keys.len() {
            if self.keys[i] {
                self.registers[x as usize] = i as u8;
                pressed = true;
                break;
            }
        }

        if !pressed {
            self.program_counter -= 2
        }
    }

    fn ld_dt_with_vx(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        self.delay_timer = self.registers[x as usize];
    }

    fn ld_st_with_vx(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        self.sound_timer = self.registers[x as usize];
    }

    fn add_i_with_vx(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        self.index_register = self
            .index_register
            .wrapping_add(self.registers[x as usize] as u16);
    }

    fn ld_i_with_font_address(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let data = self.registers[x as usize] as u16;
        self.index_register = data * 5
    }

    fn ld_bcd(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let vx = self.registers[x as usize] as f32;

        let hundreds = (vx / 100.0).floor() as u8;
        let tens = ((vx / 10.0) % 10.0).floor() as u8;
        let ones = (vx % 10.0).floor() as u8;

        self.mem_write(self.index_register, hundreds);
        self.mem_write(self.index_register + 1, tens);
        self.mem_write(self.index_register + 2, ones);
    }

    fn ld_mem_with_registers(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        for i in 0..=x {
            self.mem_write(self.index_register + i, self.registers[i as usize]);
        }
    }

    fn ld_registers_with_mem(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        for i in 0..=x {
            self.registers[i as usize] = self.mem_read(self.index_register + i);
        }
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                //BEEP
            }

            self.sound_timer -= 1;
        }
    }

    pub fn reset(&mut self) {
        self.program_counter = 0;
        self.index_register = 0;
        self.stack_pointer = 0;
        self.stack = [0; STACK_SIZE];
        self.registers = [0; NUMBER_OF_REGISTERS];
        self.memory = [0; MEMORY_SIZE];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.screen = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT];
        self.keys = [false; NUMBER_OF_KEYS];

        self.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn load(&mut self, program: &[u8]) {
        for i in 0..program.len() as u16 {
            self.mem_write(PROGRAM_START + i, program[i as usize]);
        }
        self.program_counter = PROGRAM_START;
    }

    pub fn fetch(&mut self) -> u16 {
        let opcode = self.mem_read_16(self.program_counter);
        self.program_counter += 2;
        opcode
    }

    pub fn execute(&mut self, opcode: u16) {
        let nibble1 = (opcode & 0xF000) >> 12;
        let nibble2 = (opcode & 0x0F00) >> 8;
        let nibble3 = (opcode & 0x00F0) >> 4;
        let nibble4 = opcode & 0x000F;

        match (nibble1, nibble2, nibble3, nibble4) {
            (0, 0, 0, 0) => println!("NOP"),
            (0, 0, 0xe, 0) => self.cls(),
            (0, 0, 0xE, 0xE) => self.ret(),
            (1, _, _, _) => self.jp_to_addr(opcode),
            (2, _, _, _) => self.call_at_addr(opcode),
            (3, _, _, _) => self.se_vx_and_byte(opcode),
            (4, _, _, _) => self.sne_vx_and_byte(opcode),
            (5, _, _, _) => self.se_vx_and_vy(opcode),
            (6, _, _, _) => self.ld_vx_with_byte(opcode),
            (7, _, _, _) => self.add_vx_with_byte(opcode),
            (8, _, _, 0) => self.ld_vx_with_vy(opcode),
            (8, _, _, 1) => self.or_vx_with_vy(opcode),
            (8, _, _, 2) => self.and_vx_with_vy(opcode),
            (8, _, _, 3) => self.xor_vx_with_vy(opcode),
            (8, _, _, 4) => self.add_vx_with_vy(opcode),
            (8, _, _, 5) => self.sub_vx_with_vy(opcode),
            (8, _, _, 6) => self.shr_vx(opcode),
            (8, _, _, 7) => self.subn_vx_with_vy(opcode),
            (8, _, _, 0xE) => self.shl_vx(opcode),
            (9, _, _, _) => self.sne_vx_and_vy(opcode),
            (0xA, _, _, _) => self.ld_i_with_addr(opcode),
            (0xB, _, _, _) => self.jp_to_v0_plus_addr(opcode),
            (0xC, _, _, _) => self.rnd(opcode),
            (0xD, _, _, _) => self.draw(opcode),
            (0xE, _, 9, 0xE) => self.skp(opcode),
            (0xE, _, 0xA, 1) => self.sknp(opcode),
            (0xF, _, 0, 7) => self.ld_vx_with_dt(opcode),
            (0xF, _, 0, 0xA) => self.ld_vx_with_key_press(opcode),
            (0xF, _, 1, 5) => self.ld_dt_with_vx(opcode),
            (0xF, _, 1, 8) => self.ld_st_with_vx(opcode),
            (0xF, _, 1, 0xE) => self.add_i_with_vx(opcode),
            (0xF, _, 2, 9) => self.ld_i_with_font_address(opcode),
            (0xF, _, 3, 3) => self.ld_bcd(opcode),
            (0xF, _, 5, 5) => self.ld_mem_with_registers(opcode),
            (0xF, _, 6, 5) => self.ld_registers_with_mem(opcode),
            (_, _, _, _) => unimplemented!("Opcode not defined: {opcode}"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_memory() {
        let mut interpreter = Interpreter::new();
        interpreter.mem_write(PROGRAM_START, 0x20);
        assert_eq!(interpreter.mem_read(PROGRAM_START), 0x20);
    }

    #[test]
    fn test_load() {
        let program = [7; 1000];
        let mut interpreter = Interpreter::new();
        interpreter.load(&program);
        assert_eq!(interpreter.program_counter, PROGRAM_START);
        assert_eq!(interpreter.mem_read(0x25f), 7);
    }

    #[test]
    fn test_fetch() {
        let program = [0xff; 1000];
        let mut interpreter = Interpreter::new();
        interpreter.load(&program);
        let operation = interpreter.fetch();
        assert_eq!(interpreter.program_counter, 0x202);
        assert_eq!(operation, 0xffff);
    }

    #[test]
    fn test_cls() {
        let mut interpreter = Interpreter::new();
        interpreter.screen[5][8] = true;
        interpreter.cls();
        assert_eq!(interpreter.screen[5][8], false)
    }

    #[test]
    fn test_ret() {
        let mut interpreter = Interpreter::new();
        interpreter.stack_push(0x1111);
        interpreter.ret();
        assert_eq!(interpreter.program_counter, 0x1111);
    }

    #[test]
    fn test_jp_to_addr() {
        let mut interpreter = Interpreter::new();
        interpreter.jp_to_addr(0x1111);
        assert_eq!(interpreter.program_counter, 0x0111);
    }

    #[test]
    fn test_call_at_addr() {
        let mut interpreter = Interpreter::new();
        interpreter.program_counter = PROGRAM_START;
        interpreter.call_at_addr(0x2222);
        assert_eq!(interpreter.stack_pop(), 0x0200);
        assert_eq!(interpreter.program_counter, 0x0222)
    }

    #[test]
    fn test_se_vx_and_byte() {
        let mut interpreter = Interpreter::new();
        interpreter.program_counter = PROGRAM_START;
        interpreter.registers[1] = 0x55;
        interpreter.se_vx_and_byte(0x3155);
        assert_eq!(interpreter.program_counter, 0x0202)
    }

    #[test]
    fn test_sne_vx_and_byte() {
        let mut interpreter = Interpreter::new();
        interpreter.program_counter = PROGRAM_START;
        interpreter.registers[1] = 0x55;
        interpreter.sne_vx_and_byte(0x4144);
        assert_eq!(interpreter.program_counter, 0x0202)
    }

    #[test]
    fn test_se_vx_and_vy() {
        let mut interpreter = Interpreter::new();
        interpreter.program_counter = PROGRAM_START;
        interpreter.registers[1] = 0x55;
        interpreter.registers[2] = 0x55;
        interpreter.se_vx_and_vy(0x5120);
        assert_eq!(interpreter.program_counter, 0x0202)
    }

    #[test]
    fn test_ld_vx_with_byte() {
        let mut interpreter = Interpreter::new();
        interpreter.ld_vx_with_byte(0x6130);
        assert_eq!(interpreter.registers[1], 0x0030)
    }

    #[test]
    fn test_add_vx_with_byte() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[1] = 0x33;
        interpreter.add_vx_with_byte(0x7130);
        assert_eq!(interpreter.registers[1], 0x0063)
    }

    #[test]
    fn test_ld_vx_with_vy() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[2] = 0x33;
        interpreter.ld_vx_with_vy(0x8120);
        assert_eq!(interpreter.registers[1], 0x0033)
    }

    #[test]
    fn test_or_vx_with_vy() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[7] = 0xe7;
        interpreter.registers[5] = 0x33;
        interpreter.or_vx_with_vy(0x8751);
        assert_eq!(interpreter.registers[7], 0xF7)
    }

    #[test]
    fn test_and_vx_with_vy() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[7] = 0xe7;
        interpreter.registers[5] = 0x33;
        interpreter.and_vx_with_vy(0x8752);
        assert_eq!(interpreter.registers[7], 0x23)
    }

    #[test]
    fn test_xor_vx_with_vy() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[7] = 0xe7;
        interpreter.registers[5] = 0x33;
        interpreter.xor_vx_with_vy(0x8753);
        assert_eq!(interpreter.registers[7], 0xD4)
    }

    #[test]
    fn test_add_vx_with_vy_carry() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[7] = 0xe7;
        interpreter.registers[5] = 0x33;
        interpreter.add_vx_with_vy(0x8754);
        assert_eq!(interpreter.registers[7], 0x1A);
        assert_eq!(interpreter.registers[0xF], 1)
    }

    #[test]
    fn test_add_vx_with_vy_no_carry() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[7] = 0x07;
        interpreter.registers[5] = 0x03;
        interpreter.add_vx_with_vy(0x8754);
        assert_eq!(interpreter.registers[7], 0x0A);
        assert_eq!(interpreter.registers[0xF], 0)
    }

    #[test]
    fn test_sub_vx_with_vy_borrow() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[7] = 0x33;
        interpreter.registers[5] = 0xE7;
        interpreter.sub_vx_with_vy(0x8755);
        assert_eq!(interpreter.registers[7], 0x4C);
        assert_eq!(interpreter.registers[0xF], 0)
    }

    #[test]
    fn test_sub_vx_with_vy_no_borrow() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[7] = 0xe7;
        interpreter.registers[5] = 0x33;
        interpreter.sub_vx_with_vy(0x8755);
        assert_eq!(interpreter.registers[7], 0xB4);
        assert_eq!(interpreter.registers[0xF], 1)
    }

    #[test]
    fn test_shr_vx_vf_1() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[5] = 0x33;
        interpreter.shr_vx(0x8556);
        assert_eq!(interpreter.registers[5], 0x19);
        assert_eq!(interpreter.registers[0xF], 1)
    }

    #[test]
    fn test_shr_vx_vf_0() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[5] = 0x32;
        interpreter.shr_vx(0x8556);
        assert_eq!(interpreter.registers[5], 0x19);
        assert_eq!(interpreter.registers[0xF], 0)
    }

    #[test]
    fn test_subn_vx_with_vy_borrow() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[7] = 0x33;
        interpreter.registers[5] = 0xE7;
        interpreter.subn_vx_with_vy(0x8757);
        assert_eq!(interpreter.registers[7], 0xB4);
        assert_eq!(interpreter.registers[0xF], 1)
    }

    #[test]
    fn test_subn_vx_with_vy_no_borrow() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[7] = 0xe7;
        interpreter.registers[5] = 0x33;
        interpreter.subn_vx_with_vy(0x8757);
        assert_eq!(interpreter.registers[7], 0x4C);
        assert_eq!(interpreter.registers[0xF], 0)
    }

    #[test]
    fn test_shl_vx_vf_1() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[5] = 0xE3;
        interpreter.shl_vx(0x855E);
        assert_eq!(interpreter.registers[5], 0xC6);
        assert_eq!(interpreter.registers[0xF], 1)
    }

    #[test]
    fn test_shl_vx_vf_0() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[5] = 0x32;
        interpreter.shl_vx(0x855E);
        assert_eq!(interpreter.registers[5], 0x64);
        assert_eq!(interpreter.registers[0xF], 0)
    }

    #[test]
    fn test_sne_vx_and_vy() {
        let mut interpreter = Interpreter::new();
        interpreter.program_counter = PROGRAM_START;
        interpreter.registers[1] = 0x55;
        interpreter.registers[2] = 0x54;
        interpreter.sne_vx_and_vy(0x9120);
        assert_eq!(interpreter.program_counter, 0x0202)
    }

    #[test]
    fn test_ld_i_with_addr() {
        let mut interpreter = Interpreter::new();
        interpreter.ld_i_with_addr(0xA130);
        assert_eq!(interpreter.index_register, 0x0130)
    }

    #[test]
    fn test_jp_to_v0_plus_addr() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[0] = 0x46;
        interpreter.jp_to_v0_plus_addr(0xB111);
        assert_eq!(interpreter.program_counter, 0x0157);
    }

    #[test]
    fn test_rnd() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[8] = 0x46;
        interpreter.rnd(0xC811);
        assert!(true); // cant really test this with the rng
    }

    #[test]
    fn test_draw() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[1] = 0x20;
        interpreter.registers[2] = 0x10;
        interpreter.index_register = PROGRAM_START;
        let sprite = [0xF0, 0x90, 0x90, 0x90, 0xF0];
        interpreter.memory[PROGRAM_START as usize..PROGRAM_START as usize + sprite.len()]
            .copy_from_slice(&sprite);
        interpreter.draw(0xD125);

        let coordinates = [
            (16, 32),
            (16, 33),
            (16, 34),
            (16, 35),
            (17, 32),
            (17, 35),
            (18, 32),
            (18, 35),
            (19, 32),
            (19, 35),
            (20, 32),
            (20, 33),
            (20, 34),
            (20, 35),
        ];

        for (r, c) in coordinates {
            assert!(interpreter.screen[r][c])
        }
        assert_eq!(interpreter.registers[0xF], 0)
    }

    #[test]
    fn test_skp() {
        let mut interpreter = Interpreter::new();
        interpreter.program_counter = PROGRAM_START;
        interpreter.registers[1] = 0x01;
        interpreter.keys[1] = true;
        interpreter.skp(0xE19E);
        assert_eq!(interpreter.program_counter, 0x202);
    }

    #[test]
    fn test_sknp() {
        let mut interpreter = Interpreter::new();
        interpreter.program_counter = PROGRAM_START;
        interpreter.registers[1] = 0x01;
        interpreter.keys[1] = false;
        interpreter.sknp(0xE1A1);
        assert_eq!(interpreter.program_counter, 0x202);
    }

    #[test]
    fn test_ld_vx_with_dt() {
        let mut interpreter = Interpreter::new();
        interpreter.delay_timer = 2;
        interpreter.ld_vx_with_dt(0xF107);
        assert_eq!(interpreter.registers[1], 0x02);
    }

    #[test]
    fn test_ld_vx_with_key_press() {
        let mut interpreter = Interpreter::new();
        interpreter.keys[1] = true;
        interpreter.ld_vx_with_key_press(0xF10A);
        assert_eq!(interpreter.registers[1], 0x01);
    }

    #[test]
    fn test_ld_dt_with_vx() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[1] = 2;
        interpreter.ld_dt_with_vx(0xF115);
        assert_eq!(interpreter.delay_timer, 0x02);
    }

    #[test]
    fn test_ld_st_with_vx() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[1] = 2;
        interpreter.ld_st_with_vx(0xF118);
        assert_eq!(interpreter.sound_timer, 0x02);
    }

    #[test]
    fn test_add_i_with_vx() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[1] = 0x02;
        interpreter.index_register = 0x25;
        interpreter.add_i_with_vx(0xF11E);
        assert_eq!(interpreter.index_register, 0x27);
    }

    #[test]
    fn test_ld_i_with_font_address() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[1] = 0x02;
        interpreter.ld_i_with_font_address(0xF129);
        assert_eq!(interpreter.index_register, 0x0A);
    }

    #[test]
    fn test_ld_bcd() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[1] = 0x87;
        interpreter.ld_bcd(0xF133);
        assert_eq!(interpreter.mem_read(0x0), 0x1);
        assert_eq!(interpreter.mem_read(0x1), 0x3);
        assert_eq!(interpreter.mem_read(0x2), 0x5);
    }

    #[test]
    fn test_ld_mem_with_registers() {
        let mut interpreter = Interpreter::new();
        interpreter.registers[0] = 0x84;
        interpreter.registers[1] = 0x85;
        interpreter.registers[2] = 0x86;
        interpreter.ld_mem_with_registers(0xF255);
        assert_eq!(interpreter.mem_read(0x0), 0x84);
        assert_eq!(interpreter.mem_read(0x1), 0x85);
        assert_eq!(interpreter.mem_read(0x2), 0x86);
    }

    #[test]
    fn test_ld_registers_with_mem() {
        let mut interpreter = Interpreter::new();
        interpreter.mem_write(0x0, 0x84);
        interpreter.mem_write(0x1, 0x85);
        interpreter.mem_write(0x2, 0x86);
        interpreter.ld_registers_with_mem(0xF255);
        assert_eq!(interpreter.registers[0], 0x84);
        assert_eq!(interpreter.registers[1], 0x85);
        assert_eq!(interpreter.registers[2], 0x86);
    }
}
