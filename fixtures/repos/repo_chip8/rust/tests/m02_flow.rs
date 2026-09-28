//! m02: control flow — JP, CALL, RET, stack
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
fn m02_jp() {
    let mut c = Chip8::new();
    c.load_rom(&rom(&[0x1234])).unwrap();
    c.step().unwrap();
    assert_eq!(c.pc(), 0x234);
}

#[test]
fn m02_call_sets_pc_and_stack() {
    let mut c = Chip8::new();
    // 200: CALL 204; 202: pad; 204: RET
    let mut bytes = rom(&[0x2204]);
    bytes.extend_from_slice(&[0, 0]);
    bytes.extend(rom(&[0x00EE]));
    c.load_rom(&bytes).unwrap();
    c.step().unwrap();
    assert_eq!(c.pc(), 0x204);
    assert_eq!(c.stack_depth(), 1);
}

#[test]
fn m02_ret() {
    let mut c = Chip8::new();
    let mut bytes = rom(&[0x2204]);
    bytes.extend_from_slice(&[0, 0]);
    bytes.extend(rom(&[0x00EE]));
    c.load_rom(&bytes).unwrap();
    c.step().unwrap();
    c.step().unwrap();
    assert_eq!(c.pc(), 0x202);
    assert_eq!(c.stack_depth(), 0);
}

#[test]
fn m02_nested_call_depth() {
    let mut c = Chip8::new();
    // 200: CALL 206
    // 202: RET (return target after outer)
    // 204: pad
    // 206: CALL 20A
    // 208: RET
    // 20A: RET
    let mut bytes = rom(&[0x2206, 0x00EE]);
    bytes.extend_from_slice(&[0, 0]);
    bytes.extend(rom(&[0x220A, 0x00EE, 0x00EE]));
    c.load_rom(&bytes).unwrap();
    c.step().unwrap(); // CALL 206
    assert_eq!(c.stack_depth(), 1);
    c.step().unwrap(); // CALL 20A
    assert_eq!(c.stack_depth(), 2);
    assert_eq!(c.pc(), 0x20A);
    c.step().unwrap(); // RET -> 208
    assert_eq!(c.pc(), 0x208);
    assert_eq!(c.stack_depth(), 1);
    c.step().unwrap(); // RET -> 202
    assert_eq!(c.pc(), 0x202);
    assert_eq!(c.stack_depth(), 0);
}

#[test]
fn m02_jp_then_call() {
    let mut c = Chip8::new();
    // jump to 210 which CALLs 214 then RET
    let mut bytes = vec![0u8; 0x20]; // space through 0x21x relative to 0x200
    // at 0x200: JP 210
    bytes[0] = 0x12; bytes[1] = 0x10;
    // at 0x210: CALL 214
    bytes[0x10] = 0x22; bytes[0x11] = 0x14;
    // at 0x212: 00EE (after return from call)
    bytes[0x12] = 0x00; bytes[0x13] = 0xEE;
    // at 0x214: RET
    bytes[0x14] = 0x00; bytes[0x15] = 0xEE;
    c.load_rom(&bytes).unwrap();
    c.step().unwrap();
    assert_eq!(c.pc(), 0x210);
    c.step().unwrap();
    assert_eq!(c.pc(), 0x214);
    c.step().unwrap();
    assert_eq!(c.pc(), 0x212);
}
