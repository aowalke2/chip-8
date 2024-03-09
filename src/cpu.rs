const PROGRAM_START: u16 = 0x200;

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
    stack: [u16; 16],
    registers: [u8; 16],
    memory: [u8; 4096],
    delay_timer: u8,
    sound_timer: u8,
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
        Self {
            program_counter: 0,
            index_register: 0,
            stack_pointer: 0,
            stack: [0; 16],
            registers: [0; 16],
            memory: [0; 4096],
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    fn cls(&mut self) {
        todo!()
    }

    fn ret(&mut self) {
        self.program_counter = self.stack_pop();
    }

    fn jp(&mut self, opcode: u16) {
        self.program_counter = opcode & 0x0FFF;
    }

    fn call(&mut self, opcode: u16) {
        self.stack_push(self.program_counter);
        self.program_counter = opcode & 0x0FFF;
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
            (1, _, _, _) => self.jp(opcode),
            (2, _, _, _) => self.call(opcode),
            (3, _, _, _) => println!("SE Vx, byte: {opcode}"),
            (4, _, _, _) => println!("SNE Vx, byte: {opcode}"),
            (5, _, _, _) => println!("SE Vx, Vy: {opcode}"),
            (6, _, _, _) => println!("LD Vx, byte: {opcode}"),
            (7, _, _, _) => println!("ADD Vx, byte: {opcode}"),
            (8, _, _, 0) => println!("LD Vx, Vy: {opcode}"),
            (8, _, _, 1) => println!("OR Vx, Vy: {opcode}"),
            (8, _, _, 2) => println!("AND Vx, Vy: {opcode}"),
            (8, _, _, 3) => println!("XOR Vx, Vy: {opcode}"),
            (8, _, _, 4) => println!("ADD Vx, Vy: {opcode}"),
            (8, _, _, 5) => println!("SUB Vx, Vy: {opcode}"),
            (8, _, _, 6) => println!("SHR Vx {{, Vy}}: {opcode}"),
            (8, _, _, 7) => println!("SUBN Vx, Vy: {opcode}"),
            (8, _, _, 0xE) => println!("SHL Vx {{, Vy}}: {opcode}"),
            (9, _, _, _) => println!("SNE Vx, Vy: {opcode}"),
            (0xA, _, _, _) => println!("LD I, addr: {opcode}"),
            (0xB, _, _, _) => println!("JP V0, addr: {opcode}"),
            (0xC, _, _, _) => println!("RND Vx, byte: {opcode}"),
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
    fn test_jp() {
        let mut cpu = Cpu::new();
        cpu.jp(0x1111);
        assert_eq!(cpu.program_counter, 0x0111);
    }

    #[test]
    fn test_call() {
        let mut cpu = Cpu::new();
        cpu.program_counter = PROGRAM_START;
        cpu.call(0x2222);
        assert_eq!(cpu.stack_pop(), 0x0200);
        assert_eq!(cpu.program_counter, 0x0222)
    }
}
