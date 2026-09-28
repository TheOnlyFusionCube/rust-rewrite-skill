//! m03: loads — 6xnn, 7xnn, Annn
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
fn m03_ld_vx() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x60AB])).unwrap();
    c.step().unwrap();
    assert_eq!(c.reg(0), 0xAB);
}

#[test]
fn m03_add_vx_byte() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x60AB, 0x7001])).unwrap();
    c.run(2).unwrap();
    assert_eq!(c.reg(0), 0xAC);
}

#[test]
fn m03_add_vx_wraps() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x60FF, 0x7002])).unwrap();
    c.run(2).unwrap();
    assert_eq!(c.reg(0), 0x01);
}

#[test]
fn m03_ld_i() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0xA345])).unwrap();
    c.step().unwrap();
    assert_eq!(c.i_reg(), 0x345);
}

#[test]
fn m03_combo() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x6110, 0x7105, 0xA200])).unwrap();
    c.run(3).unwrap();
    assert_eq!(c.reg(1), 0x15);
    assert_eq!(c.i_reg(), 0x200);
}
