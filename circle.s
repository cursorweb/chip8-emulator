# $0 key
# $2 x
# $3 y

    seti I, circle
    seti $1, 1

loop:
    cls
    
    draw $2, $3, 4

    seti $0, `w
    snk $0
    subf $3, $1

    seti $0, `a
    snk $0
    subf $2, $1

    seti $0, `s
    snk $0
    add $3, $1

    seti $0, `d
    snk $0
    add $2, $1

# 60 / 1 = 60 fps
    seti $f, 1
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