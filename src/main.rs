use cpu::Cpu;

pub mod cpu;

fn main() {
    let mut cpu = Cpu::new();
    cpu.decode_and_execute(0x00e0);
    cpu.decode_and_execute(0x00ee);
    cpu.decode_and_execute(0x0012);
    cpu.decode_and_execute(0x1012);
    cpu.decode_and_execute(0x2012);
    cpu.decode_and_execute(0x3012);
    cpu.decode_and_execute(0x4012);
    cpu.decode_and_execute(0x5012);
    cpu.decode_and_execute(0x6012);
    cpu.decode_and_execute(0x7012);
    cpu.decode_and_execute(0x8010);
    cpu.decode_and_execute(0x8011);
    cpu.decode_and_execute(0x8012);
    cpu.decode_and_execute(0x8013);
    cpu.decode_and_execute(0x8014);
    cpu.decode_and_execute(0x8015);
    cpu.decode_and_execute(0x8016);
    cpu.decode_and_execute(0x8017);
    cpu.decode_and_execute(0x801E);
    cpu.decode_and_execute(0x9010);
}
