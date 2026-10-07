// comments
/*
P (chiP8) is a simple c-like language.
Only two types: var* (address) and var (u8)

We create memory blocks, which the compiler organizes for us immediately after the program memory.
They must be top level, with the syntax &<name> or &*<name> to store specifically addresses
You can't mix and max, and you *shouldn't*. Use &name to store sprites or data, and &*name for rare cases.

Each local stack can have at most <=15 variables (?). Use $0 as sp or something. We will see.

Create infinite loops with loop {}. Note that programs must loop or else they crash, so end main with loop; if you must.
Also, there are while loops.

Finally, define macro constants with #define `name <expr>
all `name will be replaced with <expr>
*/

// types: &sprite address
//        u8 (everything else)

// define custom macro constant
#define `five 5

&circle {
    00011000
    00100100
    00011000
}

&x {
    00100100
    00011000
    00100100
}

&blocki {
    00111100
    00011000
    00111100
}

// allocate memory of addresses = u16
// also can do &*my_sprites {circle, x, blocki}
&*my_sprites {;3}

main {
    var x = 0;
    var y = 0;
    var wait = 50;
    var i = 0;
    
    my_sprites[0] = circle;
    my_sprites[1] = x;
    my_sprites[2] = blocki;

    loop {
        clear();
        var* player = my_sprites[i];
        draw_typed(player, x, y);

        if keydown(`w) {
            y--;
        }

        if keydown(`a) {
            x--;
        }

        if keydown(`s) {
            y++;
        }

        if keydown(`d) {
            x++;
        }

        wait--;
        if (wait == 0) {
            i = (i + 1) % sizeof my_sprites;
            wait = 50;
        }

        // wait 3 ticks is 20 fps
        sleep(3);
    }

    // you can also just loop at the end to make sure program doesn't crash:
    loop;
}

draw_typed(*sprite, x, y) {
    draw(sprite, x, y);
}