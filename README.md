# IFT6251

Here is the URL to the course repo: https://github.com/rethread-studio/algorithmic-art-course

## Depencendies

- `rust >= 1.83.0`

## How-to

To run the different art experiments:

```bash
cargo run --release --bin <bin-name>
```

where `<bin-name>` is one of:

- `triangles`
- `birds`
- `particles`

## Experiments

### triangles

For this experimentation, I drew inspiration from two main sources.

The first was [Nikolaus Gradwohl](https://www.local-guru.net/) and one of [his experimentations](https://vimeo.com/492731121). I was captivated by the mesmerizing quality of his animations, especially the way he uses computational techniques to create organic forms and fluid movements.

The second source of inspiration came from January 13th's prompt for [Genuary 2025](https://genuary.art/), created by [Heeey](https://heeey.art): "Triangles and nothing else."

I started with a single triangle as the primitive shape and explored its possibilities by manipulating its position, shape, rotation, and roll using Perlin noise. Inspired by Nikolaus's approach, I introduced the concept of slowly fading triangles to black as more are drawn, and incorporated light, translucent shapes layered over a black background to enhance the ethereal quality of the animation.

A simple, interactive menu that allows for live tweaking of the various noise multipliers and constants used in the code was also implemented, making it easier to explore and experiment with randomness. The menu includes a "Save settings" button, which prints all the current settings to the terminal for easy reference and reuse.

You may run the experiment using the following command:

```bash
cargo run --release --bin triangles
```

<!-- ## Resources -->
<!---->
<!-- - GitHub -->
<!--   - https://github.com/stars/etiennecollin/lists/ift6251 -->
<!-- - Videos -->
<!--   - [A collection of WASM demos](https://cliffle.com/p/web-demos/) -->
<!--   - [A WASM tutorial](https://www.youtube.com/watch?v=K63uBfs1K7Y) -->
