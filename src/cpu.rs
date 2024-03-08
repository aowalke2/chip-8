pub trait Memory {
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

#[derive(Debug, Clone)]
pub struct Cpu {
    program_counter: u16,
    index_register: u16,
    stack: Vec<u16>,
    registers: [u8; 16],
    memory: [u8; 4096],
}

impl Memory for Cpu {
    fn mem_read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn mem_write(&mut self, address: u16, data: u8) {
        self.memory[address as usize] = data
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        Self {
            program_counter: 0,
            index_register: 0,
            stack: vec![0; 16],
            registers: [0; 16],
            memory: [0; 4096],
        }
    }

    pub fn load(&mut self, program: &[u8]) {
        for i in 0..program.len() as u16 {
            self.mem_write(0x200 + i, program[i as usize]);
        }
        self.program_counter = 0x200;
    }

    pub fn fetch(&mut self) -> u16 {
        let opcode = self.mem_read_16(self.program_counter);
        self.program_counter += 2;
        opcode
    }

    pub fn decode_and_execute(&mut self, opcode: u16) {
        match opcode {
            0x00e0 => println!("CLS"),
            0x00ee => println!("RET"),
            opcode if opcode >> 12 == 0 => println!("SYS addr: {opcode}"),
            opcode if opcode >> 12 == 1 => println!("JP addr: {opcode}"),
            opcode if opcode >> 12 == 2 => println!("CALL addr: {opcode}"),
            opcode if opcode >> 12 == 3 => println!("SE Vx, byte: {opcode}"),
            opcode if opcode >> 12 == 4 => println!("SNE Vx, byte: {opcode}"),
            opcode if opcode >> 12 == 5 => println!("SE Vx, Vy: {opcode}"),
            opcode if opcode >> 12 == 6 => println!("LD Vx, byte: {opcode}"),
            opcode if opcode >> 12 == 7 => println!("ADD Vx, byte: {opcode}"),
            opcode if opcode >> 12 == 8 && opcode & 0xf == 0 => println!("LD Vx, Vy: {opcode}"),
            opcode if opcode >> 12 == 8 && opcode & 0xf == 1 => println!("OR Vx, Vy: {opcode}"),
            opcode if opcode >> 12 == 8 && opcode & 0xf == 2 => println!("AND Vx, Vy: {opcode}"),
            opcode if opcode >> 12 == 8 && opcode & 0xf == 3 => println!("XOR Vx, Vy: {opcode}"),
            opcode if opcode >> 12 == 8 && opcode & 0xf == 4 => println!("ADD Vx, Vy: {opcode}"),
            opcode if opcode >> 12 == 8 && opcode & 0xf == 5 => println!("SUB Vx, Vy: {opcode}"),
            opcode if opcode >> 12 == 8 && opcode & 0xf == 6 => {
                println!("SHR Vx {{, Vy}}: {opcode}")
            }
            opcode if opcode >> 12 == 8 && opcode & 0xf == 7 => println!("SUBN Vx, Vy: {opcode}"),
            opcode if opcode >> 12 == 8 && opcode & 0xf == 0xE => {
                println!("SHL Vx {{, Vy}}: {opcode}")
            }
            opcode if opcode >> 12 == 9 => println!("SNE Vx, Vy: {opcode}"),
            opcode if opcode >> 12 == 0xA => println!("LD I, addr: {opcode}"),
            opcode if opcode >> 12 == 0xB => println!("JP V0, addr: {opcode}"),
            opcode if opcode >> 12 == 0xC => println!("RND Vx, byte: {opcode}"),
            opcode if opcode >> 12 == 0xD => println!("DRW Vx, Vy, nibble: {opcode}"),
            opcode if opcode >> 12 == 0xE && opcode & 0xff == 0x9E => println!("SKP Vx: {opcode}"),
            opcode if opcode >> 12 == 0xE && opcode & 0xff == 0xA1 => println!("SKNP Vx: {opcode}"),

            _ => panic!("Opcode not defined"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_memory() {
        let mut cpu = Cpu::new();
        cpu.mem_write(0x200, 0x20);
        assert_eq!(cpu.mem_read(0x200), 0x20);
    }

    #[test]
    fn test_load() {
        let program = [7; 1000];
        let mut cpu = Cpu::new();
        cpu.load(&program);
        assert_eq!(cpu.program_counter, 0x200);
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
}
