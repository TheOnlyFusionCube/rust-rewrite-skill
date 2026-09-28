//! m04: 8xxx ALU — Cosmac VIP quirks (shift uses Vy; ADD/SUB set VF)
use chip8::Chip8;

fn rom(ops: &[u16]) -> Vec<u8> {
    let mut v = Vec::with_capacity(ops.len() * 2);
    for &w in ops {
        v.push((w >> 8) as u8);
        v.push((w & 0xff) as u8);
    }
    v
}

#[test]
fn m04_ld_vx_vy() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x61AA, 0x8010])).unwrap(); // V1=AA; LD V0,V1
    c.run(2).unwrap();
    assert_eq!(c.reg(0), 0xAA);
}

#[test]
fn m04_or_and_xor() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x60F0, 0x610F, 0x8011])).unwrap(); // OR
    c.run(3).unwrap();
    assert_eq!(c.reg(0), 0xFF);
    c.load_rom(&rom(&[0x60F0, 0x610F, 0x8012])).unwrap(); // AND
    c.run(3).unwrap();
    assert_eq!(c.reg(0), 0x00);
    c.load_rom(&rom(&[0x60FF, 0x610F, 0x8013])).unwrap(); // XOR
    c.run(3).unwrap();
    assert_eq!(c.reg(0), 0xF0);
}

#[test]
fn m04_add_carry() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x60FF, 0x6102, 0x8014])).unwrap();
    c.run(3).unwrap();
    assert_eq!(c.reg(0), 0x01);
    assert_eq!(c.reg(0xF), 1);
}

#[test]
fn m04_add_no_carry() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x6001, 0x6102, 0x8014])).unwrap();
    c.run(3).unwrap();
    assert_eq!(c.reg(0), 0x03);
    assert_eq!(c.reg(0xF), 0);
}

#[test]
fn m04_sub_borrow() {
    let mut c = Chip8::new();
    // V0=1 V1=2; SUB V0,V1 → V0=FF VF=0
    c.load_rom(&rom(&[0x6001, 0x6102, 0x8015])).unwrap();
    c.run(3).unwrap();
    assert_eq!(c.reg(0), 0xFF);
    assert_eq!(c.reg(0xF), 0);
    // V0=5 V1=3; SUB → 2 VF=1
    c.load_rom(&rom(&[0x6005, 0x6103, 0x8015])).unwrap();
    c.run(3).unwrap();
    assert_eq!(c.reg(0), 0x02);
    assert_eq!(c.reg(0xF), 1);
}

#[test]
fn m04_subn() {
    let mut c = Chip8::new();
    // V0=1 V1=5; SUBN V0,V1 → V0 = V1-V0 = 4 VF=1
    c.load_rom(&rom(&[0x6001, 0x6105, 0x8017])).unwrap();
    c.run(3).unwrap();
    assert_eq!(c.reg(0), 0x04);
    assert_eq!(c.reg(0xF), 1);
}

#[test]
fn m04_shr_cosmac_vy() {
    let mut c = Chip8::new();
    // V1=0xAA; SHR V0,V1 → V0=0x55 VF=0
    c.load_rom(&rom(&[0x61AA, 0x8016])).unwrap();
    c.run(2).unwrap();
    assert_eq!(c.reg(0), 0x55);
    assert_eq!(c.reg(0xF), 0);
    // V1=0x03; SHR V0,V1 → V0=0x01 VF=1
    c.load_rom(&rom(&[0x6103, 0x8016])).unwrap();
    c.run(2).unwrap();
    assert_eq!(c.reg(0), 0x01);
    assert_eq!(c.reg(0xF), 1);
}

#[test]
fn m04_shl_cosmac_vy() {
    let mut c = Chip8::new();
    // V1=0x80; SHL V0,V1 → V0=0 VF=1
    c.load_rom(&rom(&[0x6180, 0x801E])).unwrap();
    c.run(2).unwrap();
    assert_eq!(c.reg(0), 0x00);
    assert_eq!(c.reg(0xF), 1);
    // V1=0x41; SHL → V0=0x82 VF=0
    c.load_rom(&rom(&[0x6141, 0x801E])).unwrap();
    c.run(2).unwrap();
    assert_eq!(c.reg(0), 0x82);
    assert_eq!(c.reg(0xF), 0);
}
