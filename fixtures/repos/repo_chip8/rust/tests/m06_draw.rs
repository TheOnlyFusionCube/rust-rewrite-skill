//! m06: Dxyn — XOR draw, collision VF, wrap
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
fn m06_draw_sets_pixel() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0xA300, 0x6000, 0x6100, 0xD011])).unwrap();
    c.write_mem(0x300, &[0x80]).unwrap();
    c.run(4).unwrap();
    assert_eq!(c.framebuffer()[0], 1);
    assert_eq!(c.reg(0xF), 0);
}

#[test]
fn m06_draw_xor_collision() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0xA300, 0x6000, 0x6100, 0xD011, 0xD011])).unwrap();
    c.write_mem(0x300, &[0x80]).unwrap();
    c.run(5).unwrap();
    assert_eq!(c.framebuffer()[0], 0);
    assert_eq!(c.reg(0xF), 1);
}

#[test]
fn m06_draw_wrap_x() {
    let mut c = Chip8::new();
    // x=63, sprite 0xC0 → pixels at 63 and 0
    c.load_rom(&rom(&[0xA300, 0x603F, 0x6100, 0xD011])).unwrap();
    c.write_mem(0x300, &[0xC0]).unwrap();
    c.run(4).unwrap();
    assert_eq!(c.framebuffer()[63], 1);
    assert_eq!(c.framebuffer()[0], 1);
}

#[test]
fn m06_draw_wrap_y() {
    let mut c = Chip8::new();
    // y=31, 2-row sprite: row0 at y=31, row1 wraps to y=0
    c.load_rom(&rom(&[0xA300, 0x6000, 0x611F, 0xD012])).unwrap();
    c.write_mem(0x300, &[0x80, 0x80]).unwrap();
    c.run(4).unwrap();
    assert_eq!(c.framebuffer()[31 * 64], 1);
    assert_eq!(c.framebuffer()[0], 1);
}

#[test]
fn m06_draw_multi_bit() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0xA300, 0x6000, 0x6100, 0xD011])).unwrap();
    c.write_mem(0x300, &[0xF0]).unwrap(); // 4 pixels
    c.run(4).unwrap();
    assert_eq!(c.framebuffer()[0], 1);
    assert_eq!(c.framebuffer()[1], 1);
    assert_eq!(c.framebuffer()[2], 1);
    assert_eq!(c.framebuffer()[3], 1);
    assert_eq!(c.framebuffer()[4], 0);
}
