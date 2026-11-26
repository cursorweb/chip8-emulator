main:
    seti $0, 0                 # 0x200
    seti $1, 31                # 0x202
    seti $2, 15                # 0x204
    seti $5, 3                 # 0x206
    seti $6, 15                # 0x208
    seti $7, 253               # 0x20a
    seti $8, 15                # 0x20c
    seti I, 0x25a              # 0x20e
    rand $3, 1                 # 0x210
    snei $3, 0                 # 0x212
    seti $3, 255               # 0x214
    rand $4, 1                 # 0x216
    snei $4, 0                 # 0x218
    seti $4, 255               # 0x21a
    cls                        # 0x21c
    draw $5, $6, 4             # 0x21e
    draw $7, $8, 4             # 0x220
    draw $1, $2, 1             # 0x222
    snei $f, 1                 # 0x224
    subnf $3, $0               # 0x226
    add $1, $3                 # 0x228
    add $2, $4                 # 0x22a
    snei $1, 63                # 0x22c
    subnf $3, $0               # 0x22e
    snei $2, 31                # 0x230
    subnf $4, $0               # 0x232
    snei $1, 0                 # 0x234
    subnf $3, $0               # 0x236
    snei $2, 0                 # 0x238
    subnf $4, $0               # 0x23a
    seti $f, 5                 # 0x23c
    snk $f                     # 0x23e
    addi $6, 255               # 0x240
    seti $f, 8                 # 0x242
    snk $f                     # 0x244
    addi $6, 1                 # 0x246
    rand $f, 1                 # 0x248
    seqi $f, 1                 # 0x24a
    add $8, $4                 # 0x24c
    set $f, DELAY              # 0x24e
    seqi $f, 0                 # 0x250
    j 0x24e                    # 0x252
    seti $f, 3                 # 0x254
    set DELAY, $f              # 0x256
    j 0x21c                    # 0x258

dot:
    .byte 0x80                 # 0x25a
    .byte 0x80                 # 0x25b
    .byte 0x80                 # 0x25c
    .byte 0x80                 # 0x25d
