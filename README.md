# Sudokuist

[![Deploy to GitHub Pages](https://github.com/suomiton/sudokuist/actions/workflows/deploy-pages.yml/badge.svg)](https://github.com/suomiton/sudokuist/actions/workflows/deploy-pages.yml)
[![Build and Test](https://github.com/suomiton/sudokuist/actions/workflows/build-test.yml/badge.svg)](https://github.com/suomiton/sudokuist/actions/workflows/build-test.yml)

A minimal web-based Sudoku application built with Rust WASM backend and TypeScript frontend.

🎮 **Play Live**: [https://suomiton.github.io/sudokuist/](https://suomiton.github.io/sudokuist/)

## Features

- **Rust WASM Backend**: High-performance Sudoku generation, validation, and solving
- **TypeScript Frontend**: Type-safe UI with modern web APIs
- **IndexedDB Persistence**: Save and continue games across browser sessions
- **Clean UI**: CSS Grid-based responsive design
- **Game State Management**: URL-based game routing with UUID tracking
- **Notes System**: Add multiple possible numbers to cells as visual hints
- **Smart Input**: Keyboard shortcuts and touch-friendly controls
- **Optimized Build**: Fully minified, uglified, and compressed for optimal performance

## How to Play

### Basic Controls

- **Click a cell** to select it and open the number picker
- **Enter numbers 1-9** to fill cells
- **Clear button** or **Delete/Backspace** to empty cells
- **Hint button** to get assistance with one random cell

### Notes Feature

The notes system allows you to mark possible numbers in each cell:

- **Desktop**: Hold **Shift** + click a number to toggle it as a note
- **Mobile/Touch**: **long-press** a number to toggle it as a note
- **Keyboard**: **Shift** + number key (1-9) to toggle notes

Notes are displayed as small numbers in the top-left corner of cells and persist across sessions.

## Installation

### Prerequisites

- **Rust** (with `wasm-pack`)
- **Node.js** (with npm)

### Setup

```bash
# Install Rust and wasm-pack
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install wasm-pack

# Install dependencies
npm install
```

## Development

```bash
# Development build with dev server
npm run dev

# Optimized production build
npm run build:optimized

# Preview production build
npm run preview
```

## Project Structure

```
/
├── src/
│   ├── *.rs               # Rust WASM backend
│   ├── *.ts               # TypeScript frontend
│   └── pkg/               # Generated WASM files
├── styles/                # CSS modules
├── dist/                  # Production build output
└── index.html             # Main HTML
```

## Build Process

The project uses a highly optimized build pipeline:

- **WASM**: Compiled with release optimizations and `wasm-opt`
- **JavaScript**: Minified with Terser (aggressive settings, tree-shaking)
- **CSS**: Optimized and minified (79% size reduction)
- **HTML**: Minified with html-minifier-terser (65% size reduction)
- **Compression**: Gzip level 9 for all assets
- **Code Splitting**: Logical chunks for optimal caching

The optimized build achieves **55-79% file size reductions** across all assets.

## License

MIT
