cls
seti I, circle
seti $0, 0xF
seti $1, 0b1000
sprite $0, $1, 4
loop: j loop

circle:
    .byte 0b001100
    .byte 0b010010
    .byte 0b010010
    .byte 0b001100