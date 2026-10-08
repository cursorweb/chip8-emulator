# x $1
# y $2
# vx $3
# vy $4

    seti I, ball
    seti $1, 31
    seti $2, 15

    rand $3, 1
    snei $3, 0
    seti $3, -1

    rand $4, 1
    snei $4, 0
    seti $4, -1

loop:
    cls

    draw $1, $2, 1

    add $1, $3
    add $2, $4

    snei $1, 63
    seti $3, -1

    snei $2, 31
    seti $4, -1

    snei $1, 0
    seti $3, 1

    snei $2, 0
    seti $4, 1

    seti $f, 3
    set DELAY, $f

wait:
    set $f, DELAY
    seqi $f, 0
    j wait

    j loop

ball:
    .byte 0x80