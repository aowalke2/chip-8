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
pub struct Cpu {
    program_counter: u16,
    index_register: u16,
    stack_pointer: u8,
    stack: [u16; STACK_SIZE],
    registers: [u8; NUMBER_OF_REGISTERS],
    memory: [u8; MEMORY_SIZE],
    delay_timer: u8,
    sound_timer: u8,
    display: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    keys: [bool; NUMBER_OF_KEYS],
}

impl Memory for Cpu {
    fn mem_read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn mem_write(&mut self, address: u16, data: u8) {
        self.memory[address as usize] = data
    }
}

impl Stack for Cpu {
    fn stack_push(&mut self, address: u16) {
        self.stack[self.stack_pointer as usize] = address;
        self.stack_pointer += 1;
    }

    fn stack_pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut cpu = Self {
            program_counter: 0,
            index_register: 0,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            registers: [0; NUMBER_OF_REGISTERS],
            memory: [0; MEMORY_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            display: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            keys: [false; NUMBER_OF_KEYS],
        };

        cpu.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        return cpu;
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
        self.display = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.keys = [false; NUMBER_OF_KEYS];

        self.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    fn cls(&mut self) {
        todo!()
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
        let register = (opcode & 0x0F00) >> 8;
        let data = self.registers[register as usize];
        if data == (opcode & 0x00FF) as u8 {
            self.program_counter += 2;
        }
    }

    fn sne_vx_and_byte(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        let data = self.registers[register as usize];
        if data != (opcode & 0x00FF) as u8 {
            self.program_counter += 2;
        }
    }

    fn se_vx_and_vy(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let data_x = self.registers[x as usize];
        let data_y = self.registers[y as usize];
        if data_x == data_y {
            self.program_counter += 2;
        }
    }

    fn ld_vx_with_byte(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        self.registers[register as usize] = (opcode & 0x00FF) as u8;
    }

    fn add_vx_with_byte(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        self.registers[register as usize] += (opcode & 0x00FF) as u8;
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
        let data = self.registers[x as usize];
        self.registers[0xF] = data & 1;
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
        let data = self.registers[x as usize];
        self.registers[0xF] = (data >> 7) & 1;
        self.registers[x as usize] <<= 1;
    }

    fn sne_vx_and_vy(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let data_x = self.registers[x as usize];
        let data_y = self.registers[y as usize];
        if data_x != data_y {
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
            (0xD, _, _, _) => println!("DRW Vx, Vy, nibble: {opcode}"),
            (0xE, _, 9, 0xE) => println!("SKP Vx: {opcode}"),
            (0xE, _, 0xA, 1) => println!("SKNP Vx: {opcode}"),
            (0xF, _, 0, 7) => println!("LD Vx, DT: {opcode}"),
            (0xF, _, 0, 0xA) => println!("LD Vx, K: {opcode}"),
            (0xF, _, 1, 5) => println!("LD DT, Vx: {opcode}"),
            (0xF, _, 1, 8) => println!("LD ST, Vx: {opcode}"),
            (0xF, _, 1, 0xE) => println!("ADD I, Vx: {opcode}"),
            (0xF, _, 2, 9) => println!("LD F, Vx: {opcode}"),
            (0xF, _, 3, 3) => println!("LD B, Vx: {opcode}"),
            (0xF, _, 5, 5) => println!("LD [I], Vx: {opcode}"),
            (0xF, _, 6, 5) => println!("LD Vx, [I]: {opcode}"),
            (_, _, _, _) => unimplemented!("Opcode not defined: {opcode}"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_memory() {
        let mut cpu = Cpu::new();
        cpu.mem_write(PROGRAM_START, 0x20);
        assert_eq!(cpu.mem_read(PROGRAM_START), 0x20);
    }

    #[test]
    fn test_load() {
        let program = [7; 1000];
        let mut cpu = Cpu::new();
        cpu.load(&program);
        assert_eq!(cpu.program_counter, PROGRAM_START);
        assert_eq!(cpu.mem_read(0x25f), 7);
    }

    #[test]
    fn test_fetch() {
        let program = [0xff; 1000];
        let mut cpu = Cpu::new();
        cpu.load(&program);
        let operation = cpu.fetch();
        assert_eq!(cpu.program_counter, 0x202);
        assert_eq!(operation, 0xffff);
    }

    #[test]
    fn test_cls() {
        assert!(true);
    }

    #[test]
    fn test_ret() {
        let mut cpu = Cpu::new();
        cpu.stack_push(0x1111);
        cpu.ret();
        assert_eq!(cpu.program_counter, 0x1111);
    }

    #[test]
    fn test_jp_to_addr() {
        let mut cpu = Cpu::new();
        cpu.jp_to_addr(0x1111);
        assert_eq!(cpu.program_counter, 0x0111);
    }

    #[test]
    fn test_call_at_addr() {
        let mut cpu = Cpu::new();
        cpu.program_counter = PROGRAM_START;
        cpu.call_at_addr(0x2222);
        assert_eq!(cpu.stack_pop(), 0x0200);
        assert_eq!(cpu.program_counter, 0x0222)
    }

    #[test]
    fn test_se_vx_and_byte() {
        let mut cpu = Cpu::new();
        cpu.program_counter = PROGRAM_START;
        cpu.registers[1] = 0x55;
        cpu.se_vx_and_byte(0x3155);
        assert_eq!(cpu.program_counter, 0x0202)
    }

    #[test]
    fn test_sne_vx_and_byte() {
        let mut cpu = Cpu::new();
        cpu.program_counter = PROGRAM_START;
        cpu.registers[1] = 0x55;
        cpu.sne_vx_and_byte(0x4144);
        assert_eq!(cpu.program_counter, 0x0202)
    }

    #[test]
    fn test_se_vx_and_vy() {
        let mut cpu = Cpu::new();
        cpu.program_counter = PROGRAM_START;
        cpu.registers[1] = 0x55;
        cpu.registers[2] = 0x55;
        cpu.se_vx_and_vy(0x5120);
        assert_eq!(cpu.program_counter, 0x0202)
    }

    #[test]
    fn test_ld_vx_with_byte() {
        let mut cpu = Cpu::new();
        cpu.ld_vx_with_byte(0x6130);
        assert_eq!(cpu.registers[1], 0x0030)
    }

    #[test]
    fn test_add_vx_with_byte() {
        let mut cpu = Cpu::new();
        cpu.registers[1] = 0x33;
        cpu.add_vx_with_byte(0x7130);
        assert_eq!(cpu.registers[1], 0x0063)
    }

    #[test]
    fn test_ld_vx_with_vy() {
        let mut cpu = Cpu::new();
        cpu.registers[2] = 0x33;
        cpu.ld_vx_with_vy(0x8120);
        assert_eq!(cpu.registers[1], 0x0033)
    }

    #[test]
    fn test_or_vx_with_vy() {
        let mut cpu = Cpu::new();
        cpu.registers[7] = 0xe7;
        cpu.registers[5] = 0x33;
        cpu.or_vx_with_vy(0x8751);
        assert_eq!(cpu.registers[7], 0xF7)
    }

    #[test]
    fn test_and_vx_with_vy() {
        let mut cpu = Cpu::new();
        cpu.registers[7] = 0xe7;
        cpu.registers[5] = 0x33;
        cpu.and_vx_with_vy(0x8752);
        assert_eq!(cpu.registers[7], 0x23)
    }

    #[test]
    fn test_xor_vx_with_vy() {
        let mut cpu = Cpu::new();
        cpu.registers[7] = 0xe7;
        cpu.registers[5] = 0x33;
        cpu.xor_vx_with_vy(0x8753);
        assert_eq!(cpu.registers[7], 0xD4)
    }

    #[test]
    fn test_add_vx_with_vy_carry() {
        let mut cpu = Cpu::new();
        cpu.registers[7] = 0xe7; // 0b11100111
        cpu.registers[5] = 0x33; // 0b00110011
        cpu.add_vx_with_vy(0x8754);
        assert_eq!(cpu.registers[7], 0x1A);
        assert_eq!(cpu.registers[0xF], 1)
    }

    #[test]
    fn test_add_vx_with_vy_no_carry() {
        let mut cpu = Cpu::new();
        cpu.registers[7] = 0x07;
        cpu.registers[5] = 0x03;
        cpu.add_vx_with_vy(0x8754);
        assert_eq!(cpu.registers[7], 0x0A);
        assert_eq!(cpu.registers[0xF], 0)
    }

    #[test]
    fn test_sub_vx_with_vy_borrow() {
        let mut cpu = Cpu::new();
        cpu.registers[7] = 0x33;
        cpu.registers[5] = 0xE7;
        cpu.sub_vx_with_vy(0x8755);
        assert_eq!(cpu.registers[7], 0x4C);
        assert_eq!(cpu.registers[0xF], 0)
    }

    #[test]
    fn test_sub_vx_with_vy_no_borrow() {
        let mut cpu = Cpu::new();
        cpu.registers[7] = 0xe7;
        cpu.registers[5] = 0x33;
        cpu.sub_vx_with_vy(0x8755);
        assert_eq!(cpu.registers[7], 0xB4);
        assert_eq!(cpu.registers[0xF], 1)
    }

    #[test]
    fn test_shr_vx_vf_1() {
        let mut cpu = Cpu::new();
        cpu.registers[5] = 0x33;
        cpu.shr_vx(0x8556);
        assert_eq!(cpu.registers[5], 0x19);
        assert_eq!(cpu.registers[0xF], 1)
    }

    #[test]
    fn test_shr_vx_vf_0() {
        let mut cpu = Cpu::new();
        cpu.registers[5] = 0x32;
        cpu.shr_vx(0x8556);
        assert_eq!(cpu.registers[5], 0x19);
        assert_eq!(cpu.registers[0xF], 0)
    }

    #[test]
    fn test_subn_vx_with_vy_borrow() {
        let mut cpu = Cpu::new();
        cpu.registers[7] = 0x33;
        cpu.registers[5] = 0xE7;
        cpu.subn_vx_with_vy(0x8757);
        assert_eq!(cpu.registers[7], 0xB4);
        assert_eq!(cpu.registers[0xF], 1)
    }

    #[test]
    fn test_subn_vx_with_vy_no_borrow() {
        let mut cpu = Cpu::new();
        cpu.registers[7] = 0xe7;
        cpu.registers[5] = 0x33;
        cpu.subn_vx_with_vy(0x8757);
        assert_eq!(cpu.registers[7], 0x4C);
        assert_eq!(cpu.registers[0xF], 0)
    }

    #[test]
    fn test_shl_vx_vf_1() {
        let mut cpu = Cpu::new();
        cpu.registers[5] = 0xE3;
        cpu.shl_vx(0x855E);
        assert_eq!(cpu.registers[5], 0xC6);
        assert_eq!(cpu.registers[0xF], 1)
    }

    #[test]
    fn test_shl_vx_vf_0() {
        let mut cpu = Cpu::new();
        cpu.registers[5] = 0x32;
        cpu.shl_vx(0x855E);
        assert_eq!(cpu.registers[5], 0x64);
        assert_eq!(cpu.registers[0xF], 0)
    }

    #[test]
    fn test_sne_vx_and_vy() {
        let mut cpu = Cpu::new();
        cpu.program_counter = PROGRAM_START;
        cpu.registers[1] = 0x55;
        cpu.registers[2] = 0x54;
        cpu.sne_vx_and_vy(0x9120);
        assert_eq!(cpu.program_counter, 0x0202)
    }

    #[test]
    fn test_ld_i_with_addr() {
        let mut cpu = Cpu::new();
        cpu.ld_i_with_addr(0xA130);
        assert_eq!(cpu.index_register, 0x0130)
    }

    #[test]
    fn test_jp_to_v0_plus_addr() {
        let mut cpu = Cpu::new();
        cpu.registers[0] = 0x46;
        cpu.jp_to_v0_plus_addr(0xB111);
        assert_eq!(cpu.program_counter, 0x0157);
    }

    #[test]
    fn test_rnd() {
        let mut cpu = Cpu::new();
        cpu.registers[8] = 0x46;
        cpu.rnd(0xC811);
        assert_eq!(cpu.registers[8], 0);
    }
}
