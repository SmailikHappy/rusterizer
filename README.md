## About the Project

It is a simple rasterizer written in Rust, developed as part of [BUas (university)](https://www.buas.nl/) masterclasses.
My goal was to understand the basics of rasterization algorithms, while acquiring proficiency in the Rust programming language.

My primary focus was to research and learn new stuff, as well the project has been originated at the beginning of my journey as a programmer. As a result, **the code quality does not reflect my current programming skills.**

## Key features

- Loading a `.gltf` model
- Applying `.jpg` texture onto the loaded model
- Triangle rasterization using [Bresenham's algorithm](https://en.wikipedia.org/wiki/Bresenham's_line_algorithm)
- Free camera movement (<kbd>W</kbd><kbd>A</kbd><kbd>S</kbd><kbd>D</kbd> and mouse)

## Changing of assets

In the file `src/main.rs` you can change the path to the assets you would like to see in the application:

- *line 26:* you can change the path to your `.gltf` model
```rust
const MESH_PATH: &str = "assets/helmet.gltf";
```

- *line 27:* you can change the path to your texture that will be applied to the model
```rust
const TEXT_PATH: &str = "assets/albedo.jpg";
```

> I recommend to place your assets in `/assets` folder of the repo.

## Usage

### Prerequisites

Rust programming language should be installed on your system to build and test the code. Visit [Rust's official website](https://www.rust-lang.org/) for installation instructions.

### Launching

1. Clone the repository to your local machine
2. Open cmd and navigate to the project directory 
3. Build and run the project using Cargo:

```sh
cargo build
```
```sh
cargo run
```

___
Thank you for reading this.
Enjoy!
gg