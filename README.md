# IFT6251

Here is the URL to the course repo: https://github.com/rethread-studio/algorithmic-art-course

## TOC

<!-- vim-markdown-toc GFM -->

- [Dependencies](#dependencies)
- [How-to](#how-to)
- [Experiments](#experiments)
  - [birds](#birds)
    - [Running](#running)
  - [particles](#particles)
    - [Running](#running-1)
  - [triangles](#triangles)
    - [Interaction](#interaction)
    - [Running](#running-2)
  - [mandelbrot](#mandelbrot)
    - [Interaction](#interaction-1)
    - [Running](#running-3)
    - [Next Steps](#next-steps)

<!-- vim-markdown-toc -->

## Dependencies

- `rust >= 1.83.0`

## How-to

To run the different art experiments:

```bash
cargo run --release --bin <bin-name>
```

where `<bin-name>` is one of:

- `birds`
- `mandelbrot`
- `particles`
- `triangles`

## Experiments

### birds

This experiments simply models the flocking behaviour of birds.

#### Running

You may run the experiment using the following command:

```bash
cargo run --release --bin birds
```

---

### particles

This experiments is a simple particle simulator featuring non-elastic collisions.
Each particle has a mass that changes its color and size.

#### Running

You may run the experiment using the following command:

```bash
cargo run --release --bin particles
```

---

### triangles

For this experimentation, I drew inspiration from two main sources.

The first was [Nikolaus Gradwohl](https://www.local-guru.net/) and one of [his experimentations](https://vimeo.com/492731121). I was captivated by the mesmerizing quality of his animations, especially the way he uses computational techniques to create organic forms and fluid movements.

The second source of inspiration came from January 13th's prompt for [Genuary 2025](https://genuary.art/), created by [Heeey](https://heeey.art): "Triangles and nothing else."

I started with a single triangle as the primitive shape and explored its possibilities by manipulating its position, shape, rotation, and roll using Perlin noise. Inspired by Nikolaus's approach, I introduced the concept of slowly fading triangles to black as more are drawn, and incorporated light, translucent shapes layered over a black background to enhance the ethereal quality of the animation.

A simple, interactive menu that allows for live tweaking of the various noise multipliers and constants used in the code was also implemented, making it easier to explore and experiment with randomness. The menu includes a "Save settings" button, which prints all the current settings to the terminal for easy reference and reuse.

#### Interaction

- **`S` Key** → Save the current frame
- **`Q` Key** → Quit

#### Running

You may run the experiment using the following command:

```bash
cargo run --release --bin triangles
```

---

### mandelbrot

For this experiment, I drew inspiration from a simple but profound mathematical theme: **recursion**. The Mandelbrot set is the ultimate recursive fractal; each point in the complex plane is tested against a rule that feeds back into itself, again and again, to determine its fate.

From a single formula, an infinite landscape emerges. This experiment is an invitation to explore recursion visually, to get lost in the infinite depth of the Mandelbrot set, and to uncover new patterns hidden within the chaos.

This experiment is an attempt to not only visualize this fractal but to explore it interactively, manipulating its parameters in real-time and experimenting with alternative ways to render its intricate structure. The project also expands on traditional Mandelbrot rendering by introducing **"subtrajectory" visualization**, a technique that maps each iteration of a complex series back to screen-space, revealing the path of individual points as they evolve.

Another key feature is the ability to **selectively render** either the points **inside** or **outside** the set, offering a different perspective on the fractal’s structure.

#### Interaction

- **Arrow Keys** → Move the viewport
- **`+` / `-`** → Zoom in/out
- **Mouse Scroll** → Zoom dynamically
- **`S` Key** → Save the current frame
- **`Return` Key** → Force redraw
- **`Q` Key** → Quit

#### Running

You may run the experiment using the following command:

```bash
cargo run --release --bin mandelbrot
```

#### Next Steps

The next step would be to write a shader to compute the mandelbrot set for the screen. Right now, the render is only real-time for a low iteration count and low sub-pixel count. Computing the set on the GPU would make the code a lot faster.

<!-- ## Resources -->
<!---->
<!-- - GitHub -->
<!--   - https://github.com/stars/etiennecollin/lists/ift6251 -->
<!-- - Videos -->
<!--   - [A collection of WASM demos](https://cliffle.com/p/web-demos/) -->
<!--   - [A WASM tutorial](https://www.youtube.com/watch?v=K63uBfs1K7Y) -->
