cls                            # 0x200
ldi I, 0x22a                   # 0x202
ldi V0, 12                     # 0x204
ldi V1, 8                      # 0x206
sprite V0 V1 15                # 0x208
addi V0, 9                     # 0x20a
ldi I, 0x239                   # 0x20c
sprite V0 V1 15                # 0x20e
ldi I, 0x248                   # 0x210
addi V0, 8                     # 0x212
sprite V0 V1 15                # 0x214
addi V0, 4                     # 0x216
ldi I, 0x257                   # 0x218
sprite V0 V1 15                # 0x21a
addi V0, 8                     # 0x21c
ldi I, 0x266                   # 0x21e
sprite V0 V1 15                # 0x220
addi V0, 8                     # 0x222
ldi I, 0x275                   # 0x224
sprite V0 V1 15                # 0x226
j 0x228                        # 0x228


.word 0xff00                   # 0x22a
.word 0xff00                   # 0x22c
.word 0x3c00                   # 0x22e
.word 0x3c00                   # 0x230
.word 0x3c00                   # 0x232
.word 0x3c00                   # 0x234
.word 0xff00                   # 0x236
.word 0xffff                   # 0x238
.word 0x00ff                   # 0x23a
.word 0x0038                   # 0x23c
.word 0x003f                   # 0x23e
.word 0x003f                   # 0x240
.word 0x0038                   # 0x242
.word 0x00ff                   # 0x244
.word 0x00ff                   # 0x246
.word 0x8000                   # 0x248
.word 0xe000                   # 0x24a
.word 0xe000                   # 0x24c
.word 0x8000                   # 0x24e
.word 0x8000                   # 0x250
.word 0xe000                   # 0x252
.word 0xe000                   # 0x254
.word 0x80f8                   # 0x256
.word 0x00fc                   # 0x258
.word 0x003e                   # 0x25a
.word 0x003f                   # 0x25c
.word 0x003b                   # 0x25e
.word 0x0039                   # 0x260
.word 0x00f8                   # 0x262
.word 0x00f8                   # 0x264
.word 0x0300                   # 0x266
.word 0x0700                   # 0x268
.word 0x0f00                   # 0x26a
.word 0xbf00                   # 0x26c
.word 0xfb00                   # 0x26e
.word 0xf300                   # 0x270
.word 0xe300                   # 0x272
.word 0x43e0                   # 0x274
.word 0x00e0                   # 0x276
.word 0x0080                   # 0x278
.word 0x0080                   # 0x27a
.word 0x0080                   # 0x27c
.word 0x0080                   # 0x27e
.word 0x00e0                   # 0x280
.word 0x00e0                   # 0x282
