//! m07: timers + keypad
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
fn m07_ld_dt_and_read() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x6005, 0xF015, 0xF107])).unwrap(); // DT=5; V1=DT
    c.run(3).unwrap();
    assert_eq!(c.delay_timer(), 5);
    assert_eq!(c.reg(1), 5);
}

#[test]
fn m07_tick_timers() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x6003, 0xF015, 0xF018])).unwrap();
    c.run(3).unwrap();
    assert_eq!(c.delay_timer(), 3);
    assert_eq!(c.sound_timer(), 3);
    c.tick_timers();
    assert_eq!(c.delay_timer(), 2);
    assert_eq!(c.sound_timer(), 2);
    c.tick_timers();
    c.tick_timers();
    c.tick_timers();
    assert_eq!(c.delay_timer(), 0);
    assert_eq!(c.sound_timer(), 0);
}

#[test]
fn m07_skp_key_pressed() {
    let mut c = Chip8::new();
    // V0=2; SKP V0; LD V1,1; LD V2,2
    c.load_rom(&rom(&[0x6002, 0xE09E, 0x6101, 0x6202])).unwrap();
    c.set_key(2, true);
    c.run(3).unwrap();
    assert_eq!(c.reg(1), 0);
    assert_eq!(c.reg(2), 2);
}

#[test]
fn m07_sknp_key_up() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x6002, 0xE0A1, 0x6101, 0x6202])).unwrap();
    c.set_key(2, false);
    c.run(3).unwrap();
    assert_eq!(c.reg(1), 0); // skipped because key not pressed
    assert_eq!(c.reg(2), 2);
}

#[test]
fn m07_fx0a_wait_then_store() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0xF00A, 0x6101])).unwrap();
    c.step().unwrap();
    assert_eq!(c.pc(), 0x200);
    assert_eq!(c.reg(0), 0);
    // still waiting
    c.step().unwrap();
    assert_eq!(c.pc(), 0x200);
    c.set_key(0xC, true);
    c.step().unwrap();
    assert_eq!(c.reg(0), 0xC);
    assert_eq!(c.pc(), 0x202);
    c.step().unwrap();
    assert_eq!(c.reg(1), 1);
}
