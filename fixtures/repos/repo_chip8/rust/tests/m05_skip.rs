//! m05: skips + JP V0 (Bnnn)
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
fn m05_se_vx_byte() {
    let mut c = Chip8::new();
    // V0=5; SE V0,5; LD V1,1  — should skip LD
    c.load_rom(&rom(&[0x6005, 0x3005, 0x6101, 0x6202])).unwrap();
    c.run(3).unwrap();
    assert_eq!(c.reg(1), 0); // skipped
    assert_eq!(c.reg(2), 2);
}

#[test]
fn m05_sne_vx_byte() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x6005, 0x4005, 0x6101, 0x6202])).unwrap();
    c.run(4).unwrap(); // SE does not skip; both LDs run
    assert_eq!(c.reg(1), 1); // not skipped
    assert_eq!(c.reg(2), 2);
}

#[test]
fn m05_se_vx_vy() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x6007, 0x6107, 0x5010, 0x6201, 0x6302])).unwrap();
    c.run(4).unwrap();
    assert_eq!(c.reg(2), 0);
    assert_eq!(c.reg(3), 2);
}

#[test]
fn m05_sne_vx_vy() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x6007, 0x6108, 0x9010, 0x6201, 0x6302])).unwrap();
    c.run(4).unwrap();
    assert_eq!(c.reg(2), 0); // skipped LD V2
    assert_eq!(c.reg(3), 2);
}

#[test]
fn m05_jp_v0() {
    let mut c = Chip8::new();
    // V0=0x10; B200 → PC = 0x200+0x10 = 0x210
    // place LD V1,0xAA at 0x210
    let mut bytes = vec![0u8; 0x20];
    bytes[0] = 0x60; bytes[1] = 0x10; // LD V0,10
    bytes[2] = 0xB2; bytes[3] = 0x00; // JP V0,200
    bytes[0x10] = 0x61; bytes[0x11] = 0xAA; // at 0x210
    c.load_rom(&bytes).unwrap();
    c.run(3).unwrap();
    assert_eq!(c.pc(), 0x212);
    assert_eq!(c.reg(1), 0xAA);
}
