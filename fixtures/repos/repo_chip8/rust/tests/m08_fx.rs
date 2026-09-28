//! m08: FX misc — ADD I, font, BCD, reg dump/load (I increments)
use chip8::{Chip8, FONT_ADDR};

fn rom(ops: &[u16]) -> Vec<u8> {
    let mut v = Vec::with_capacity(ops.len() * 2);
    for &w in ops {
        v.push((w >> 8) as u8);
        v.push((w & 0xff) as u8);
    }
    v
}

#[test]
fn m08_add_i() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0xA100, 0x6005, 0xF01E])).unwrap();
    c.run(3).unwrap();
    assert_eq!(c.i_reg(), 0x105);
}

#[test]
fn m08_ld_font() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x600A, 0xF029])).unwrap();
    c.run(2).unwrap();
    assert_eq!(c.i_reg(), (FONT_ADDR + 0xA * 5) as u16);
    let sprite = c.mem_slice(c.i_reg() as usize, 5).unwrap();
    assert_eq!(sprite[0], 0xF0); // 'A' glyph starts with F0
}

#[test]
fn m08_bcd() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x60FE, 0xA400, 0xF033])).unwrap();
    c.run(3).unwrap();
    assert_eq!(c.mem_slice(0x400, 3).unwrap(), &[2, 5, 4]);
}

#[test]
fn m08_reg_dump_fx55() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x6001, 0x6102, 0x6203, 0xA500, 0xF255])).unwrap();
    c.run(5).unwrap();
    assert_eq!(c.mem_slice(0x500, 3).unwrap(), &[1, 2, 3]);
    assert_eq!(c.i_reg(), 0x503); // Cosmac VIP: I += x+1
}

#[test]
fn m08_reg_load_fx65() {
    let mut c = Chip8::new();
    c.write_mem(0x500, &[9, 8, 7]).unwrap();
    c.load_rom(&rom(&[0xA500, 0xF265])).unwrap();
    // load_rom clears mem program area but write_mem after:
    c.load_rom(&rom(&[0xA500, 0xF265])).unwrap();
    c.write_mem(0x500, &[9, 8, 7]).unwrap();
    c.run(2).unwrap();
    assert_eq!(c.reg(0), 9);
    assert_eq!(c.reg(1), 8);
    assert_eq!(c.reg(2), 7);
    assert_eq!(c.i_reg(), 0x503);
}
