# chip8 — mini CHIP-8 core (Python reference)

Behavioral oracle for the Rust port under `../rust/`. **Not** a Game Boy emulator
and **not** RIIR Bench / SameBoy. Inspired at mini-repo scale by RIIR Bench's
Game Boy DNA real-port slice ([rewritebench.com](https://rewritebench.com)).

## Quirk profile: Cosmac VIP / chip8-test-suite friendly

| Behavior | Choice |
|----------|--------|
| Shift `8xy6` / `8xyE` | **Vy → Vx** (shift Vy, store in Vx; VF = shifted-out bit of Vy) |
| `8xy4` ADD | VF = carry (1 if result > 0xFF) |
| `8xy5` SUB | VF = NOT borrow (1 if Vx ≥ Vy before) |
| `8xy7` SUBN | VF = NOT borrow (1 if Vy ≥ Vx before) |
| `Bnnn` JP V0 | PC = nnn + V0 (not Bxnn) |
| `Dxyn` draw | XOR sprites; **wrap** x/y modulo 64×32; VF=1 on any pixel erase |
| Timers | `tick_timers()` decrements delay/sound by 1 if >0 (CPU `step` does not) |
| `Fx0A` | Blocks (PC stays) until a key is pressed; then Vx = key index |
| Font | Digits 0–F at **0x50–0x9F** (5 bytes each) |
| `Fx55`/`Fx65` | **I increments** by x+1 after (Cosmac VIP) |
| Load | ROM bytes start at **0x200**; PC reset to 0x200 |

## API

```python
from chip8 import Chip8
cpu = Chip8()
cpu.load_rom(rom_bytes)   # at 0x200
cpu.step()                # one opcode; may raise Chip8Error
cpu.run(n)                # n steps
cpu.mem_slice(start, len)
cpu.framebuffer()         # list/bytes length 64*32, 0 or 1
cpu.set_key(i, pressed)   # i in 0..15
cpu.keys()
cpu.delay_timer() / cpu.sound_timer()
cpu.reg(vx) / cpu.set_reg(vx, val)   # test helpers
cpu.pc() / cpu.i_reg() / cpu.stack_depth()
cpu.tick_timers()
```

## Self-check

```bash
cd source && PYTHONPATH=. python3 -m tests.selfcheck
```
