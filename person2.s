    j 0x20a                    # 0x200

person:
    .byte 0x70                 # 0x202
    .byte 0x70                 # 0x203
    .byte 0x20                 # 0x204
    .byte 0x70                 # 0x205
    .byte 0xa8                 # 0x206
    .byte 0x20                 # 0x207
    .byte 0x50                 # 0x208
    .byte 0x50                 # 0x209

main:
    rand $1, 31                # 0x20a
    rand $2, 15                # 0x20c
    seti I, 0x202              # 0x20e
    draw $1, $2, 8             # 0x210
    draw $1, $2, 8             # 0x212
    seti $0, 5                 # 0x214
    snk $0                     # 0x216
    addi $2, 255               # 0x218
    seti $0, 8                 # 0x21a
    snk $0                     # 0x21c
    addi $2, 1                 # 0x21e
    seti $0, 7                 # 0x220
    snk $0                     # 0x222
    addi $1, 255               # 0x224
    seti $0, 9                 # 0x226
    snk $0                     # 0x228
    addi $1, 1                 # 0x22a
    draw $1, $2, 8             # 0x22c
    set $f, DELAY              # 0x22e
    seqi $f, 0                 # 0x230
    j 0x22e                    # 0x232
    seti $f, 3                 # 0x234
    set DELAY, $f              # 0x236
    j 0x212                    # 0x238
