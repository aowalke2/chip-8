use cpu::Cpu;

pub mod cpu;

fn main() {
    let mut cpu = Cpu::new();
    cpu.execute(0x00e0);
    cpu.execute(0x00ee);
    cpu.execute(0x0012);
    cpu.execute(0x1012);
    cpu.execute(0x2012);
    cpu.execute(0x3012);
    cpu.execute(0x4012);
    cpu.execute(0x5012);
    cpu.execute(0x6012);
    cpu.execute(0x7012);
    cpu.execute(0x8010);
    cpu.execute(0x8011);
    cpu.execute(0x8012);
    cpu.execute(0x8013);
    cpu.execute(0x8014);
    cpu.execute(0x8015);
    cpu.execute(0x8016);
    cpu.execute(0x8017);
    cpu.execute(0x801E);
    cpu.execute(0x9010);
    cpu.execute(0xA012);
    cpu.execute(0xB012);
    cpu.execute(0xC012);
    cpu.execute(0xD012);
    cpu.execute(0xE09E);
    cpu.execute(0xE1A1);
    cpu.execute(0xF007);
    cpu.execute(0xF00A);
    cpu.execute(0xF015);
    cpu.execute(0xF018);
    cpu.execute(0xF01E);
    cpu.execute(0xF029);
    cpu.execute(0xF033);
    cpu.execute(0xF055);
    cpu.execute(0xF065);
}
