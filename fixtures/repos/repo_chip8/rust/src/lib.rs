//! CHIP-8 core port (Cosmac VIP quirks). Match `source/` behavior.
//! Do NOT edit, weaken, or delete files under `tests/`.
//!
//! Quirks (must match Python README):
//! - Shift 8xy6/8xyE: Vy → Vx; VF = shifted-out bit of Vy
//! - ADD/SUB/SUBN set VF carry/borrow as Cosmac VIP
//! - Bnnn: PC = nnn + V0
//! - Dxyn: XOR, wrap x/y, VF on collision
//! - Font at 0x50; Fx55/Fx65 increment I by x+1
//! - tick_timers() decrements DT/ST; step() does not
//! - Fx0A waits until set_key; then stores key index in Vx

#![allow(unused_variables)]

pub const MEM_SIZE: usize = 4096;
pub const FB_W: usize = 64;
pub const FB_H: usize = 32;
pub const FB_SIZE: usize = FB_W * FB_H;
pub const FONT_ADDR: usize = 0x50;
pub const PROG_START: usize = 0x200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Chip8Error {
    RomTooLarge,
    BadOpcode(u16),
    StackUnderflow,
    StackOverflow,
    MemOutOfRange,
    BadKey,
    Other(String),
}

impl std::fmt::Display for Chip8Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Chip8Error {}

pub struct Chip8 {
    // Implement fields as needed.
}

impl Chip8 {
    pub fn new() -> Self {
        todo!("Chip8::new — 4K mem, font at 0x50, PC=0x200, clear fb/regs/timers")
    }

    /// Load ROM bytes at 0x200; reset PC/regs/timers/fb (keep font).
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), Chip8Error> {
        todo!("load_rom")
    }

    /// Execute one opcode (or service Fx0A wait).
    pub fn step(&mut self) -> Result<(), Chip8Error> {
        todo!("step")
    }

    /// Run `n` steps.
    pub fn run(&mut self, n: u32) -> Result<(), Chip8Error> {
        todo!("run")
    }

    pub fn mem_slice(&self, start: usize, len: usize) -> Result<&[u8], Chip8Error> {
        todo!("mem_slice")
    }

    /// Length FB_SIZE; each entry 0 or 1.
    pub fn framebuffer(&self) -> &[u8] {
        todo!("framebuffer")
    }

    pub fn set_key(&mut self, i: u8, pressed: bool) {
        todo!("set_key")
    }

    pub fn key(&self, i: u8) -> bool {
        todo!("key")
    }

    pub fn delay_timer(&self) -> u8 {
        todo!("delay_timer")
    }

    pub fn sound_timer(&self) -> u8 {
        todo!("sound_timer")
    }

    pub fn reg(&self, vx: u8) -> u8 {
        todo!("reg")
    }

    pub fn set_reg(&mut self, vx: u8, val: u8) {
        todo!("set_reg")
    }

    pub fn pc(&self) -> u16 {
        todo!("pc")
    }

    pub fn i_reg(&self) -> u16 {
        todo!("i_reg")
    }

    pub fn stack_depth(&self) -> usize {
        todo!("stack_depth")
    }

    /// Decrement delay and sound timers by 1 if > 0.
    pub fn tick_timers(&mut self) {
        todo!("tick_timers")
    }

    /// Test helper: write bytes into memory.
    pub fn write_mem(&mut self, addr: usize, data: &[u8]) -> Result<(), Chip8Error> {
        todo!("write_mem")
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}
