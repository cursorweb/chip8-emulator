rand $1, 31                    # 0x200
rand $2, 15                    # 0x202
seti I, person                 # 0x204
sprite $1, $2, 8               # 0x206
sprite $1, $2, 8               # 0x208
seti $0, 5                     # 0x20a
snk $0                         # 0x20c
addi $2, 255                   # 0x20e
seti $0, 8                     # 0x210
snk $0                         # 0x212
addi $2, 1                     # 0x214
seti $0, 7                     # 0x216
snk $0                         # 0x218
addi $1, 255                   # 0x21a
seti $0, 9                     # 0x21c
snk $0                         # 0x21e
addi $1, 1                     # 0x220
sprite $1, $2, 8               # 0x222
set $f, DELAY                  # 0x224
seqi $f, 0                     # 0x226
j 0x224                        # 0x228
seti $f, 3                     # 0x22a
set DELAY, $f                  # 0x22c
j 0x208                        # 0x22e

person:
    .byte 0x70                 # 0x230
    .byte 0x70
    .byte 0x20
    .byte 0x70
    .byte 0xA8
    .byte 0x20
    .byte 0x50
    .byte 0x50
