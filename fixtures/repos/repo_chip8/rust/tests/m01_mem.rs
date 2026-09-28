//! m01: memory + 00E0 clear
use chip8::{Chip8, FONT_ADDR, PROG_START};

fn rom(ops: &[u16]) -> Vec<u8> {
    let mut v = Vec::with_capacity(ops.len() * 2);
    for &w in ops {
        v.push((w >> 8) as u8);
        v.push((w & 0xff) as u8);
    }
    v
}

#[test]
fn m01_font_present_at_0x50() {
    let c = Chip8::new();
    let font = c.mem_slice(FONT_ADDR, 80).expect("font slice");
    assert_eq!(font.len(), 80);
    // digit 0 first byte
    assert_eq!(font[0], 0xF0);
    // digit 1 first byte
    assert_eq!(font[5], 0x20);
}

#[test]
fn m01_load_rom_at_0x200() {
    let mut c = Chip8::new();
    let r = rom(&[0x00E0]);
    c.load_rom(&r).unwrap();
    assert_eq!(c.pc(), PROG_START as u16);
    assert_eq!(c.mem_slice(0x200, 2).unwrap(), &[0x00, 0xE0]);
}

#[test]
fn m01_write_mem_roundtrip() {
    let mut c = Chip8::new();
    c.write_mem(0x300, &[0xDE, 0xAD]).unwrap();
    assert_eq!(c.mem_slice(0x300, 2).unwrap(), &[0xDE, 0xAD]);
}

#[test]
fn m01_cls_clears_framebuffer() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x00E0])).unwrap();
    // force a pixel on via write after load — use draw path? poke fb via draw:
    // Draw one pixel then CLS.
    c.write_mem(0x300, &[0x80]).unwrap();
    // prepend more ops by reloading: A300, 6000, 6100, D011, 00E0
    c.load_rom(&rom(&[0xA300, 0x6000, 0x6100, 0xD011, 0x00E0])).unwrap();
    c.write_mem(0x300, &[0x80]).unwrap();
    c.run(4).unwrap();
    assert_eq!(c.framebuffer()[0], 1);
    c.step().unwrap(); // CLS
    assert!(c.framebuffer().iter().all(|&p| p == 0));
}

#[test]
fn m01_load_rom_resets_pc() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x1234])).unwrap();
    c.step().unwrap();
    assert_eq!(c.pc(), 0x234);
    c.load_rom(&rom(&[0x00E0])).unwrap();
    assert_eq!(c.pc(), 0x200);
}
