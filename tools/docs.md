# CHIP-8 Assembly Reference

Custom CHIP-8 assembly syntax.

Registers use `V0`-`VF`. Hexadecimal values use `0x` prefixes. You can also do `0b` and just `nn` as decimal.

## Control Flow
| Opcode | Assembly     | Description              |
| ------ | ------------ | ------------------------ |
| `00E0` | `cls`        | Clear the display        |
| `00EE` | `ret`        | Return from subroutine (see `call`) |
| `1NNN` | `j NNN`      | Jump to address `NNN`    |
| `2NNN` | `call NNN`   | Call subroutine at `NNN` |
| `BNNN` | `jri0 NNN`   | Jump to `NNN + V0` (special command, unused) |
| `BXNN` | `jri Vx, NN` | Jump to `NN + Vx`        |

## Conditional Skips
| Opcode | Assembly      | Description                                      |
| ------ | ------------- | ------------------------------------------------ |
| `3XNN` | `seqi Vx, NN` | Skip next instruction if `Vx == NN`              |
| `4XNN` | `snei Vx, NN` | Skip next instruction if `Vx != NN`              |
| `5XY0` | `seq Vx, Vy`  | Skip next instruction if `Vx == Vy`              |
| `9XY0` | `sne Vx, Vy`  | Skip next instruction if `Vx != Vy`              |
| `EX9E` | `sk Vx`       | Skip next instruction if key `Vx` is pressed     |
| `EXA1` | `snk Vx`      | Skip next instruction if key `Vx` is not pressed |

## Register Operations
| Opcode | Assembly       | Description                                           |
| ------ | -------------- | ----------------------------------------------------- |
| `6XNN` | `seti Vx, NN`  | Set `Vx = NN`                                         |
| `7XNN` | `addi Vx, NN`  | Add `NN` to `Vx`                                      |
| `8XY0` | `set Vx, Vy`   | Set `Vx = Vy`                                         |
| `8XY1` | `or Vx, Vy`    | `Vx = Vx \| Vy`                                       |
| `8XY2` | `and Vx, Vy`   | `Vx = Vx & Vy`                                        |
| `8XY3` | `xor Vx, Vy`   | `Vx = Vx ^ Vy`                                        |
| `8XY4` | `add Vx, Vy`   | `Vx = Vx + Vy`                                        |
| `8XY5` | `subf Vx, Vy`  | `Vx = Vx - Vy`, set `VF` to no-borrow flag (ie `Vx >= Vy`) |
| `8XY6` | `srlf Vx`      | Shift `Vx` right, `VF` gets shifted-out bit           |
| `8XY6` | `srlf Vx, Vy`  | Shift `Vy` right into `Vx`, `VF` gets shifted-out bit |
| `8XY7` | `subnf Vx, Vy` | `Vx = Vy - Vx`, set `VF` to no-borrow flag (ie `Vy >= Vx`) |
| `8XYE` | `sllf Vx`      | Shift `Vx` left, `VF` gets shifted-out bit            |
| `8XYE` | `sllf Vx, Vy`  | Shift `Vy` left into `Vx`, `VF` gets shifted-out bit  |

The two-operand shift forms depend on the `shift_y` configuration.

## Index Register
| Opcode | Assembly        | Description     |
| ------ | --------------- | --------------- |
| `ANNN` | `seti I, 0xNNN` | Set `I = NNN`   |
| `FX1E` | `add I, Vx`     | Add `Vx` to `I` |

## Timers and Input
| Opcode | Assembly        | Description                               |
| ------ | --------------- | ----------------------------------------- |
| `FX07` | `set Vx, DELAY` | Load delay timer into `Vx`                |
| `FX0A` | `set Vx, KEY`   | Wait for a key press and store it in `Vx` |
| `FX15` | `set DELAY, Vx` | Set delay timer to `Vx`                   |
| `FX18` | `set SOUND, Vx` | Set sound timer to `Vx`                   |

## Random Number Generation
| Opcode | Assembly      | Description                        |
| ------ | ------------- | ---------------------------------- |
| `CXNN` | `rand Vx, NN` | Set `Vx` to random byte `AND` `NN` |

## Graphics
| Opcode | Assembly           | Description                                                                   |
| ------ | ------------------ | ----------------------------------------------------------------------------- |
| `DXYN` | `sprite Vx, Vy, N` | Draw an 8×N sprite at `(Vx, Vy)` from memory at `I`; `VF` indicates collision |

A sprite is encoded as one byte per row:

```text
1 byte = 8 pixels
N bytes = N rows
```

For example:

```text
FF = 11111111
00 = 00000000
18 = 00011000
```

produces:

```text
########
         
   ##
```

## Memory
| Opcode | Assembly          | Description                                                         |
| ------ | ----------------- | ------------------------------------------------------------------- |
| `FX29` | `set I, FONT[Vx]` | Set `I` to the address of the font sprite for the character in `Vx` |
| `FX33` | `bcd Vx`          | Store the decimal digits of `Vx` at `I`, `I+1`, and `I+2`           |
| `FX55` | `set [I], Vx`     | Store `V0`..=`Vx` starting at memory address `I`                    |
| `FX65` | `set Vx, [I]`     | Load `V0`..=`Vx` from memory starting at `I`                        |

For example:

```text
set [I], V3
```

stores:

```text
M[I]     = V0
M[I + 1] = V1
M[I + 2] = V2
M[I + 3] = V3
```

## Raw Data
| Directive      | Description                      |
| -------------- | -------------------------------- |
| `.word 0xNNNN` | Emit/preserve a raw 16-bit value |
