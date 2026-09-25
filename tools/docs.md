# CHIP-8 Assembly Reference

Custom CHIP-8 assembly syntax.

Registers use `$0`-`$F`. Hexadecimal values use `0x` prefixes. You can also do `0b` and just `nn` as decimal.

## Control Flow
| Opcode | Assembly     | Description              |
| ------ | ------------ | ------------------------ |
| `00E0` | `cls`        | Clear the display        |
| `00EE` | `ret`        | Return from subroutine (see `call`) |
| `1NNN` | `j NNN`      | Jump to address `NNN`    |
| `2NNN` | `call NNN`   | Call subroutine at `NNN` |
| `BNNN` | `jri0 NNN`   | Jump to `NNN + $0` (special command, unused) |
| `BXNN` | `jri $x, NN` | Jump to `NN + $x`        |

## Conditional Skips
| Opcode | Assembly      | Description                                      |
| ------ | ------------- | ------------------------------------------------ |
| `3XNN` | `seqi $x, NN` | Skip next instruction if `$x == NN`              |
| `4XNN` | `snei $x, NN` | Skip next instruction if `$x != NN`              |
| `5XY0` | `seq $x, $y`  | Skip next instruction if `$x == $y`              |
| `9XY0` | `sne $x, $y`  | Skip next instruction if `$x != $y`              |
| `EX9E` | `sk $x`       | Skip next instruction if key `$x` is pressed     |
| `EXA1` | `snk $x`      | Skip next instruction if key `$x` is not pressed |

## Register Operations
| Opcode | Assembly       | Description                                           |
| ------ | -------------- | ----------------------------------------------------- |
| `6XNN` | `seti $x, NN`  | Set `$x = NN`                                         |
| `7XNN` | `addi $x, NN`  | Add `$x = $x + NN`                                    |
| `8XY0` | `set $x, $y`   | Set `$x = $y`                                         |
| `8XY1` | `or $x, $y`    | `$x = $x \| $y`                                       |
| `8XY2` | `and $x, $y`   | `$x = $x & $y`                                        |
| `8XY3` | `xor $x, $y`   | `$x = $x ^ $y`                                        |
| `8XY4` | `add $x, $y`   | `$x = $x + $y`                                        |
| `8XY5` | `subf $x, $y`  | `$x = $x - $y`, set `$F` to no-borrow flag (ie `$x >= $y`) |
| `8XY6` | `srlf $x`      | Shift `$x` right, `$F` gets shifted-out bit (special unused) |
| `8XY6` | `srlf $x, $y`  | Shift `$y` right into `$x`, `$F` gets shifted-out bit |
| `8XY7` | `subnf $x, $y` | `$x = -($x - $y) = $y - $x`, set `$F` to no-borrow flag (ie `$y >= $x`) |
| `8XYE` | `sllf $x`      | Shift `$x` left, `$F` gets shifted-out bit (special unused) |
| `8XYE` | `sllf $x, $y`  | Shift `$y` left into `$x`, `$F` gets shifted-out bit  |

The two-operand shift forms depend on the `shift_y` configuration.

## Index Register
| Opcode | Assembly        | Description     |
| ------ | --------------- | --------------- |
| `ANNN` | `seti I, 0xNNN` | Set `I = NNN`   |
| `FX1E` | `add I, $x`     | Add `$x` to `I` |

## Timers and Input
| Opcode | Assembly        | Description                               |
| ------ | --------------- | ----------------------------------------- |
| `FX07` | `set $x, DELAY` | Load delay timer into `$x`                |
| `FX0A` | `set $x, KEY`   | Wait for a key press and store it in `$x` |
| `FX15` | `set DELAY, $x` | Set delay timer to `$x`                   |
| `FX18` | `set SOUND, $x` | Set sound timer to `$x`                   |

## Random Number Generation
| Opcode | Assembly      | Description                        |
| ------ | ------------- | ---------------------------------- |
| `CXNN` | `rand $x, NN` | Set `$x` to random byte `AND` `NN` |

## Graphics
| Opcode | Assembly           | Description                                                                   |
| ------ | ------------------ | ----------------------------------------------------------------------------- |
| `DXYN` | `sprite $x, $y, N` | Draw an 8×N sprite at `($x, $y)` from memory at `I`; `$F` indicates collision |

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
| `FX29` | `set I, FONT[$x]` | Set `I` to the address of the font sprite for the character in `$x` |
| `FX33` | `bcd $x`          | Store the decimal digits of `$x` at `I`, `I+1`, and `I+2`           |
| `FX55` | `set [I], $x`     | Store `$0`..=`$x` starting at memory address `I`                    |
| `FX65` | `set $x, [I]`     | Load `$0`..=`$x` from memory starting at `I`                        |

For example:

```text
set [I], $3
```

stores:

```text
M[I]     = $0
M[I + 1] = $1
M[I + 2] = $2
M[I + 3] = $3
```

## Raw Data
| Directive      | Description             |
| -------------- | ----------------------- |
| `.word 0xNNNN` | Emit a raw 16-bit value |
| `.byte 0xNN`   | Emit a raw 8-bit value  |

## Labels
```
    seti I, square
    seti V0, 6
    seti V1, 7
    sprite V0, V1, 2
loop: j loop

square:
    .word 0b00011000
    .word 0b00011000
```
