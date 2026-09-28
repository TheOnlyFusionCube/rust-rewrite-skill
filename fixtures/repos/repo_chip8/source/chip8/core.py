"""CHIP-8 interpreter core (Cosmac VIP / chip8-test-suite friendly quirks)."""

from __future__ import annotations

from typing import List

MEM_SIZE = 4096
FB_W = 64
FB_H = 32
FB_SIZE = FB_W * FB_H
STACK_MAX = 16
FONT_ADDR = 0x50
PROG_START = 0x200

# Standard CHIP-8 hex font (0–F), 5 bytes each → 0x50..0x9F
FONT: bytes = bytes(
    [
        0xF0, 0x90, 0x90, 0x90, 0xF0,  # 0
        0x20, 0x60, 0x20, 0x20, 0x70,  # 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0,  # 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0,  # 3
        0x90, 0x90, 0xF0, 0x10, 0x10,  # 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0,  # 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0,  # 6
        0xF0, 0x10, 0x20, 0x40, 0x40,  # 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0,  # 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0,  # 9
        0xF0, 0x90, 0xF0, 0x90, 0x90,  # A
        0xE0, 0x90, 0xE0, 0x90, 0xE0,  # B
        0xF0, 0x80, 0x80, 0x80, 0xF0,  # C
        0xE0, 0x90, 0x90, 0x90, 0xE0,  # D
        0xF0, 0x80, 0xF0, 0x80, 0xF0,  # E
        0xF0, 0x80, 0xF0, 0x80, 0x80,  # F
    ]
)


class Chip8Error(Exception):
    """Illegal opcode, stack underflow/overflow, or bad memory access."""


class Chip8:
    def __init__(self) -> None:
        self._mem = bytearray(MEM_SIZE)
        self._v = [0] * 16
        self._i = 0
        self._pc = PROG_START
        self._sp = 0
        self._stack = [0] * STACK_MAX
        self._delay = 0
        self._sound = 0
        self._fb = [0] * FB_SIZE
        self._keys = [False] * 16
        self._waiting_key: int | None = None  # Vx index when blocked on Fx0A
        # Install font
        self._mem[FONT_ADDR : FONT_ADDR + len(FONT)] = FONT

    def load_rom(self, rom: bytes) -> None:
        if len(rom) > MEM_SIZE - PROG_START:
            raise Chip8Error("ROM too large")
        # Clear program area but keep font
        self._mem[PROG_START:] = b"\x00" * (MEM_SIZE - PROG_START)
        self._mem[PROG_START : PROG_START + len(rom)] = rom
        self._pc = PROG_START
        self._sp = 0
        self._i = 0
        self._v = [0] * 16
        self._delay = 0
        self._sound = 0
        self._fb = [0] * FB_SIZE
        self._waiting_key = None

    def step(self) -> None:
        if self._waiting_key is not None:
            vx = self._waiting_key
            for ki, pressed in enumerate(self._keys):
                if pressed:
                    self._v[vx] = ki & 0xFF
                    self._waiting_key = None
                    self._pc = (self._pc + 2) & 0xFFFF  # advance past Fx0A
                    return
            return  # still waiting; PC still on Fx0A

        if self._pc + 1 >= MEM_SIZE:
            raise Chip8Error(f"PC out of range: {self._pc:#x}")
        op = (self._mem[self._pc] << 8) | self._mem[self._pc + 1]
        self._pc = (self._pc + 2) & 0xFFFF
        self._exec(op)

    def run(self, n: int) -> None:
        for _ in range(n):
            self.step()

    def _exec(self, op: int) -> None:
        nnn = op & 0x0FFF
        nn = op & 0x00FF
        n = op & 0x000F
        x = (op >> 8) & 0x0F
        y = (op >> 4) & 0x0F
        hi = (op >> 12) & 0x0F

        if op == 0x00E0:
            self._fb = [0] * FB_SIZE
            return
        if op == 0x00EE:
            if self._sp == 0:
                raise Chip8Error("stack underflow on RET")
            self._sp -= 1
            self._pc = self._stack[self._sp]
            return
        if hi == 0x0:
            raise Chip8Error(f"unknown 0xxx opcode {op:#06x}")
        if hi == 0x1:
            self._pc = nnn
            return
        if hi == 0x2:
            if self._sp >= STACK_MAX:
                raise Chip8Error("stack overflow on CALL")
            self._stack[self._sp] = self._pc
            self._sp += 1
            self._pc = nnn
            return
        if hi == 0x3:
            if self._v[x] == nn:
                self._pc = (self._pc + 2) & 0xFFFF
            return
        if hi == 0x4:
            if self._v[x] != nn:
                self._pc = (self._pc + 2) & 0xFFFF
            return
        if hi == 0x5 and n == 0:
            if self._v[x] == self._v[y]:
                self._pc = (self._pc + 2) & 0xFFFF
            return
        if hi == 0x6:
            self._v[x] = nn
            return
        if hi == 0x7:
            self._v[x] = (self._v[x] + nn) & 0xFF
            return
        if hi == 0x8:
            self._alu(x, y, n)
            return
        if hi == 0x9 and n == 0:
            if self._v[x] != self._v[y]:
                self._pc = (self._pc + 2) & 0xFFFF
            return
        if hi == 0xA:
            self._i = nnn
            return
        if hi == 0xB:
            self._pc = (nnn + self._v[0]) & 0xFFFF
            return
        if hi == 0xC:
            raise Chip8Error("Cxnn RNG not required in this mini-core")
        if hi == 0xD:
            self._draw(x, y, n)
            return
        if hi == 0xE:
            if nn == 0x9E:
                if self._keys[self._v[x] & 0x0F]:
                    self._pc = (self._pc + 2) & 0xFFFF
                return
            if nn == 0xA1:
                if not self._keys[self._v[x] & 0x0F]:
                    self._pc = (self._pc + 2) & 0xFFFF
                return
            raise Chip8Error(f"unknown Exnn {op:#06x}")
        if hi == 0xF:
            self._fx(x, nn)
            return
        raise Chip8Error(f"unknown opcode {op:#06x}")

    def _alu(self, x: int, y: int, n: int) -> None:
        vx, vy = self._v[x], self._v[y]
        if n == 0x0:
            self._v[x] = vy
        elif n == 0x1:
            self._v[x] = vx | vy
        elif n == 0x2:
            self._v[x] = vx & vy
        elif n == 0x3:
            self._v[x] = vx ^ vy
        elif n == 0x4:
            s = vx + vy
            self._v[0xF] = 1 if s > 0xFF else 0
            self._v[x] = s & 0xFF
        elif n == 0x5:
            self._v[0xF] = 1 if vx >= vy else 0
            self._v[x] = (vx - vy) & 0xFF
        elif n == 0x6:
            # Cosmac VIP: shift Vy into Vx
            self._v[0xF] = vy & 0x1
            self._v[x] = (vy >> 1) & 0xFF
        elif n == 0x7:
            self._v[0xF] = 1 if vy >= vx else 0
            self._v[x] = (vy - vx) & 0xFF
        elif n == 0xE:
            self._v[0xF] = (vy >> 7) & 0x1
            self._v[x] = (vy << 1) & 0xFF
        else:
            raise Chip8Error(f"unknown 8xyN n={n:#x}")

    def _draw(self, x: int, y: int, n: int) -> None:
        x0 = self._v[x] % FB_W
        y0 = self._v[y] % FB_H
        self._v[0xF] = 0
        for row in range(n):
            byte = self._mem[(self._i + row) & 0xFFF]
            py = (y0 + row) % FB_H
            for bit in range(8):
                if byte & (0x80 >> bit):
                    px = (x0 + bit) % FB_W
                    idx = py * FB_W + px
                    if self._fb[idx]:
                        self._v[0xF] = 1
                    self._fb[idx] ^= 1

    def _fx(self, x: int, nn: int) -> None:
        if nn == 0x07:
            self._v[x] = self._delay & 0xFF
        elif nn == 0x0A:
            # Rewind PC onto this opcode and wait
            self._pc = (self._pc - 2) & 0xFFFF
            self._waiting_key = x
        elif nn == 0x15:
            self._delay = self._v[x]
        elif nn == 0x18:
            self._sound = self._v[x]
        elif nn == 0x1E:
            self._i = (self._i + self._v[x]) & 0xFFFF
        elif nn == 0x29:
            digit = self._v[x] & 0x0F
            self._i = FONT_ADDR + digit * 5
        elif nn == 0x33:
            val = self._v[x]
            self._mem[self._i & 0xFFF] = val // 100
            self._mem[(self._i + 1) & 0xFFF] = (val // 10) % 10
            self._mem[(self._i + 2) & 0xFFF] = val % 10
        elif nn == 0x55:
            for i in range(x + 1):
                self._mem[(self._i + i) & 0xFFF] = self._v[i]
            # Cosmac VIP: I increments past last written
            self._i = (self._i + x + 1) & 0xFFFF
        elif nn == 0x65:
            for i in range(x + 1):
                self._v[i] = self._mem[(self._i + i) & 0xFFF]
            self._i = (self._i + x + 1) & 0xFFFF
        else:
            raise Chip8Error(f"unknown Fxnn nn={nn:#x}")

    def tick_timers(self) -> None:
        if self._delay > 0:
            self._delay -= 1
        if self._sound > 0:
            self._sound -= 1

    def mem_slice(self, start: int, length: int) -> bytes:
        if start < 0 or length < 0 or start + length > MEM_SIZE:
            raise Chip8Error("mem_slice out of range")
        return bytes(self._mem[start : start + length])

    def framebuffer(self) -> List[int]:
        return list(self._fb)

    def set_key(self, i: int, pressed: bool) -> None:
        if not 0 <= i <= 15:
            raise Chip8Error("key index out of range")
        self._keys[i] = bool(pressed)

    def keys(self) -> List[bool]:
        return list(self._keys)

    def delay_timer(self) -> int:
        return self._delay

    def sound_timer(self) -> int:
        return self._sound

    def reg(self, vx: int) -> int:
        return self._v[vx & 0x0F]

    def set_reg(self, vx: int, val: int) -> None:
        self._v[vx & 0x0F] = val & 0xFF

    def pc(self) -> int:
        return self._pc

    def i_reg(self) -> int:
        return self._i

    def stack_depth(self) -> int:
        return self._sp

    def write_mem(self, addr: int, data: bytes) -> None:
        """Test helper: poke bytes into memory."""
        end = addr + len(data)
        if addr < 0 or end > MEM_SIZE:
            raise Chip8Error("write_mem out of range")
        self._mem[addr:end] = data
