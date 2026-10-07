# $0 key
# $2 x
# $3 y

    seti I, circle
    seti $1, 1

loop:
    cls
    
    draw $2, $3, 4

    seti $0, 5  # W
    snk $0
    subf $3, $1

    seti $0, 7  # A
    snk $0
    subf $2, $1

    seti $0, 8  # S
    snk $0
    add $3, $1

    seti $0, 9  # D
    snk $0
    add $2, $1

# 60 / 2 = 30 fps
    seti $f, 2
    set DELAY, $f
    
wait:
    set $f, DELAY
    seqi $f, 0
    j wait

    j loop

circle:
    .byte 0b00011000
    .byte 0b00100100
    .byte 0b00100100
    .byte 0b00011000