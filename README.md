# Calculator

A Windows 11-style GUI calculator built with Rust using [eframe/egui](https://github.com/emilk/egui).

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)

## Features

- **Standard calculator operations**: addition, subtraction, multiplication, division
- **Scientific functions**: reciprocal (1/x), square (x²), square root (√x)
- **Utility buttons**: percent (%), clear (C), clear entry (CE), backspace (DEL), sign toggle (+/-)
- **Expression display**: shows the current operation above the result
- **Collapsible history panel**: expand/collapse with the "History >>" button; shows the last 10 operations
- **Persistent history**: history is saved to disk and restored across sessions
- **Division by zero handling**: displays "Error" with a descriptive message
- **Dark theme**: Windows 11-inspired color palette
- **420 Easter egg**: when a calculation result hits 420, enjoy an animated cannabis leaf with rainbow colors, rising smoke particles, and the iconic Snoop Dogg audio clip

## Building

Requires Rust 1.70+ and the MSVC toolchain on Windows.

```sh
cargo build --release
```

The binary is output to `target/release/calc.exe`.

## Running

```sh
./target/release/calc.exe
```

Or double-click `calc.exe` from the file explorer. The console window is hidden in release builds.

## Testing

The project includes unit tests and integration tests.

```sh
cargo test
```

## Layout

```
+--------------------------------------+
|                          History >>   |
|                        3 + 5 =       |
|                               8      |
|                                      |
|  [ %  ] [ CE ] [  C ] [ DEL ]       |
|  [1/x ] [ x2 ] [ Vx ] [  /  ]      |
|  [  7  ] [  8 ] [  9 ] [  x  ]      |
|  [  4  ] [  5 ] [  6 ] [  -  ]      |
|  [  1  ] [  2 ] [  3 ] [  +  ]      |
|  [ +/- ] [  0 ] [  . ] [  =  ]      |
+--------------------------------------+
```

## Project Structure

```
calculator/
  Cargo.toml
  src/
    lib.rs          # Core calculator logic, history persistence, and unit tests
    main.rs         # GUI application using eframe/egui
  assets/
    blaze_mono.wav  # Embedded 420 Easter egg audio clip
  tests/
    integration_test.rs  # Integration tests
```

- **`src/lib.rs`** contains the `CalcApp` struct with all calculator operations (digit input, operators, compute, clear, backspace, sign toggle, percent), the `HistoryEntry` type, number formatting, and history file I/O. All unit tests live here.
- **`src/main.rs`** wraps `CalcApp` in a thin GUI shell that implements `eframe::App`, rendering the display, button grid, collapsible history panel, and the 420 Easter egg animation with embedded audio.
- **`tests/integration_test.rs`** exercises the calculator library through multi-step operation sequences.

## Tech Stack

- **[eframe](https://crates.io/crates/eframe)** v0.29 -- native app framework
- **[egui](https://crates.io/crates/egui)** -- immediate mode GUI
- **[rodio](https://crates.io/crates/rodio)** v0.19 -- audio playback
- **glow** (OpenGL) renderer for broad GPU compatibility
