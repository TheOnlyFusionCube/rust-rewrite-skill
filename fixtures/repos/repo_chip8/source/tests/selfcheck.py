"""Smoke tests for the Python CHIP-8 reference (must pass before shipping fixture)."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from chip8 import Chip8
from chip8.core import FONT, FONT_ADDR


def op(*words: int) -> bytes:
    out = bytearray()
    for w in words:
        out.append((w >> 8) & 0xFF)
        out.append(w & 0xFF)
    return bytes(out)


def check(name: str, cond: bool) -> None:
    if not cond:
        raise AssertionError(f"FAIL: {name}")
    print(f"  ok {name}")


def main() -> None:
    print("chip8 Python selfcheck")
    c = Chip8()
    check("font installed", c.mem_slice(FONT_ADDR, 80) == FONT)

    c.load_rom(op(0x00E0))
    c._fb[0] = 1
    c.step()
    check("CLS clears fb", all(p == 0 for p in c.framebuffer()))

    c = Chip8()
    c.load_rom(op(0x2204) + b"\x00\x00" + op(0x00EE))
    c.step()
    check("CALL sets pc", c.pc() == 0x204)
    check("CALL stack depth 1", c.stack_depth() == 1)
    c.step()
    check("RET returns", c.pc() == 0x202)
    check("RET stack depth 0", c.stack_depth() == 0)

    c = Chip8()
    c.load_rom(op(0x1234))
    c.step()
    check("JP", c.pc() == 0x234)

    c = Chip8()
    c.load_rom(op(0x60AB, 0x7001, 0xA300))
    c.run(3)
    check("LD Vx / ADD", c.reg(0) == 0xAC)
    check("LD I", c.i_reg() == 0x300)

    c = Chip8()
    c.load_rom(op(0x60FF, 0x6102, 0x8014))
    c.run(3)
    check("ADD carry VF", c.reg(0xF) == 1 and c.reg(0) == 1)

    c = Chip8()
    c.load_rom(op(0x61AA, 0x8016))
    c.run(2)
    check("SHR Cosmac Vy", c.reg(0) == 0x55 and c.reg(0xF) == 0)

    c = Chip8()
    sprite = bytes([0x80])
    c.load_rom(op(0xA300, 0x6000, 0x6100, 0xD011, 0xD011))
    c.write_mem(0x300, sprite)
    c.run(5)
    check("draw collision VF", c.reg(0xF) == 1)
    check("draw xor off", c.framebuffer()[0] == 0)

    c = Chip8()
    c.load_rom(op(0xA300, 0x603F, 0x6100, 0xD011))
    c.write_mem(0x300, bytes([0xC0]))
    c.run(4)
    check("draw wrap x", c.framebuffer()[63] == 1 and c.framebuffer()[0] == 1)

    c = Chip8()
    c.load_rom(op(0x6005, 0xF015, 0xF007))
    c.run(2)
    check("LD DT", c.delay_timer() == 5)
    c.tick_timers()
    check("tick_timers", c.delay_timer() == 4)
    c.step()
    check("LD Vx, DT", c.reg(0) == 4)

    c = Chip8()
    c.load_rom(op(0xF00A, 0x6001))
    c.step()
    check("Fx0A waits", c.pc() == 0x200 and c.reg(0) == 0)
    c.set_key(7, True)
    c.step()
    check("Fx0A stores key", c.reg(0) == 7 and c.pc() == 0x202)

    c = Chip8()
    c.load_rom(op(0x60FE, 0xA400, 0xF033))
    c.run(3)
    check("BCD", c.mem_slice(0x400, 3) == bytes([2, 5, 4]))

    c = Chip8()
    c.load_rom(op(0x600A, 0xF029))
    c.run(2)
    check("LD F", c.i_reg() == FONT_ADDR + 0xA * 5)

    # Fx55 increments I (Cosmac VIP)
    c = Chip8()
    c.load_rom(op(0x6001, 0x6102, 0xA500, 0xF155))
    c.run(4)
    check("Fx55 mem", c.mem_slice(0x500, 2) == bytes([1, 2]))
    check("Fx55 I bump", c.i_reg() == 0x502)

    print("ALL SELFCHECK PASSED")


if __name__ == "__main__":
    main()
