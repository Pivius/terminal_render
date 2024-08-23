# Terminal Render

## Overview

This project provides a Rust-based implementation for capturing, processing, and displaying graphics on Windows using the Windows Graphics Capture API. 
The project includes custom handling of pixel data, image processing algorithms, and terminal-based rendering.

## Table of Contents

- [Terminal Render](#terminal-render)
  - [Overview](#overview)
  - [Table of Contents](#table-of-contents)
  - [Installation](#installation)
    - [Dependencies](#dependencies)
  - [Usage](#usage)
  - [Coding Style](#coding-style)

## Installation

To build and run this project, you'll need to have Rust installed. You can install Rust from [here](https://rustup.rs).

### Dependencies

The project relies on the following crates:
- `windows_capture` for window capturing and frame handling.
- `crossterm` for terminal manipulation.
- `winapi` and `kernel32` for Windows API interactions.

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
windows_capture = "0.1"
crossterm = "0.26"
winapi = { version = "0.3", features = ["wincon", "winnls"] }
kernel32 = "0.1"
```

## Usage

1. Build the project
    ```
    cargo build
    ```
2. Run the project
    ```
    cargo run
    ```

Currently you need to specify the window title of the window you want to capture within main.rs.
And any postprocessing needs to be applied onto the pixel map in canvas.

## Coding Style
- Follow Rust's standard coding style guidelines. This includes conventions like using `snake_case` for variable and function names, `CamelCase` for structs and enums, and `UPPER_SNAKE_CASE` for constants.
- Indentation: Use 4 spaces per indentation.
- Line Length: Keep lines under 120 characters.
- Error Handling: Use `Result` and `Option` types to propagate errors using the `?` operator where applicable.
- Code Structure: Keep functions short and focused, and group related functions and structs in modules.