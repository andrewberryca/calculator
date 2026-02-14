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

## Layout

```
┌──────────────────────────────────────┐
│                          History >>  │
│                        3 + 5 =       │
│                               8      │
│                                      │
│  [ %  ] [ CE ] [  C ] [ DEL ]       │
│  [1/x ] [ x² ] [ √x ] [  ÷ ]       │
│  [  7  ] [  8 ] [  9 ] [  × ]       │
│  [  4  ] [  5 ] [  6 ] [  − ]       │
│  [  1  ] [  2 ] [  3 ] [  + ]       │
│  [ +/- ] [  0 ] [  . ] [  = ]       │
└──────────────────────────────────────┘
```

## Tech Stack

- **[eframe](https://crates.io/crates/eframe)** v0.29 — native app framework
- **[egui](https://crates.io/crates/egui)** — immediate mode GUI
- **glow** (OpenGL) renderer for broad GPU compatibility
