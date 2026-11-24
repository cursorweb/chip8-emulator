rand $1, 31                    # 0x200
rand $2, 15                    # 0x202
draw $1, $2, 8                 # 0x204
draw $1, $2, 8                 # 0x206
seti $0, 5                     # 0x208
sk $0                          # 0x20a
addi $2, 255                   # 0x20c
seti $0, 8                     # 0x20e
sk $0                          # 0x210
addi $2, 1                     # 0x212
seti $0, 7                     # 0x214
sk $0                          # 0x216
addi $1, 255                   # 0x218
seti $0, 9                     # 0x21a
sk $0                          # 0x21c
addi $1, 1                     # 0x21e
draw $1, $2, 8                 # 0x220
set $f, DELAY                  # 0x222
snei $f, 0                     # 0x224
j 0x222                        # 0x226
seti $f, 3                     # 0x228
set DELAY, $f                  # 0x22a
j 0x206                        # 0x22c
