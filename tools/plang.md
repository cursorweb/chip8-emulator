# P (chiP8)
P (chiP8) is a simple C-like programming language designed for writing programs that compile to CHIP-8.

It provides abstractions for variables (aka registers), functions, control flow, static memory, and compile-time constants while leaving most low-level memory management to the compiler.

## Types
P has two types:

* `var`: an unsigned 8-bit value (`u8`), ranging from `0` to `255`.
* `var*`: a memory address.

Addresses are `u16`. Ordinary values are `u8`.

## Program Structure
A P program begins with `main`:

```c
main {
    // program
}
```

Additional functions are declared by writing their name followed by their parameters and a body:
```c
draw_player(*sprite, x, y) {
    draw(sprite, x, y);
}
```

Functions are called using normal function-call syntax:
```c
draw_player(player, x, y);
```

## Variables
Variables (always local) are declared with `var` or `var*`:
```js
var x = 10;
var* player = circle;

// assignment
x = 20;
player = sprite2;
```

Variables must be initialized when they are declared.

P may limit functions to have at most 15 local variables or less (for corresponding to each register, and then reserved for compiler operations). Maybe change in future.

## Static Memory
A byte-oriented block is declared with `&`:

```c
&circle {
    0b00011000,
    0b00100100,
    0b00011000,
}
```

This creates a block of memory (the compiler automatically places it immediately after the program's code), and binds `circle` to the address of that block of memory.

A block containing addresses is declared with `&*`:
```c
&*my_sprites {;3}
```
`my_sprites` reserves space for three addresses (each address takes up **2** bytes).

Address blocks can also be initialized directly:
```c
&*my_sprites {
    circle,
    player,
    enemy,
}
```

`&name` and `&*name` represent different layouts and should not be mixed. `&name` should generally be used for sprites and ordinary data, while `&*name` is intended for collections of addresses.

## Memory Access
An address can be indexed to access the corresponding byte or address element:

```c
var* player = circle;

player[0] = 5;
var x = player[1];
```

For an address block:

```c
var* sprite = my_sprites[0];
```

The size of each element is determined by the block's type.

P does not currently expose arbitrary pointer arithmetic. Indexed access is the intended way to work with memory.

## Control Flow

### `if`
```c
if x > 10 {
    x = 0;
} else if x > 5 {
    x--;
} else {
    x++;
}
```

### `while`
```c
while x != 0 {
    x--;
}
```

### Infinite loops
`loop` creates an infinite loop:

```c
loop {
    draw(player, x, y);
    wait(3);
}
```

An empty `loop;` is also valid:

```c
loop;
```

This creates a busy infinite loop.

CHIP-8 programs must not simply fall through the end of `main`, since there is no normal program termination mechanism. Use `loop;` if you need.

## Functions
Functions may take `u8` or address, differentiated by `*`:
```c
draw_typed(*sprite, x, y) {
    draw(sprite, x, y);
}
```

The first parameter is an address, while `x` and `y` are `u8`.

Functions can return values using `return`:

```c
&*addresses {...}

add(x, y) var {
    return x + y;
}

addr(i) var* {
    return addresses[i];
}

empty() {}
```

## Standard "Library"
P provides a small set of built-in operations for interacting with the CHIP-8 system.

### `draw`
```c
draw(*sprite, x, y);
```

### `clear`
```c
clear();
```

### `keydown`
```c
keydown(k);
```

Probably use with a predefined macro: ``keydown(`a)``

### `sleep`
```c
sleep(t);
```

Since CHIP-8 timers run at 60 Hz, `sleep(3)` at the end of the main loop will give you 60 / 3 = 20 fps.

### `sizeof`
`sizeof` is a language keyword rather than an ordinary function. It obtains the size (ie length) of a statically allocated object:

```c
&*sprites {;3};
var i = sizeof sprites; // i = 3
```

## Compile-Time Constants
P supports simple compile-time macro constants using `#define`. (Simple textual replacement)

```c
#define `five 5
#define `mask (0xF)

main {
    var x = `five;
    x = x & `mask;

    loop;
}
```
