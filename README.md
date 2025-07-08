# Sudokuist

A minimal web-based Sudoku application built with Rust WASM backend and TypeScript frontend.

## Features

- **Rust WASM Backend**: High-performance Sudoku generation, validation, and solving
- **TypeScript Frontend**: Type-safe UI with modern web APIs
- **IndexedDB Persistence**: Save and continue games across browser sessions
- **Clean UI**: CSS Grid-based responsive design
- **Game State Management**: URL-based game routing with UUID tracking

## Project Structure

```
/
├── index.html              # Main HTML file
├── styles/
│   ├── reset.css          # CSS reset
│   └── styles.css         # Main styles with CSS Grid board
├── src/
│   ├── app.ts             # TypeScript frontend application
│   └── lib.rs             # Rust WASM backend
├── pkg/                   # Generated WASM files (after build)
├── dist/                  # Built frontend files
└── build files (Cargo.toml, package.json, etc.)
```

## Prerequisites

- **Rust** (with `wasm-pack`)
- **Node.js** (with npm/yarn)

## Installation

1. Install Rust and wasm-pack:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install wasm-pack
```

2. Install Node.js dependencies:

```bash
npm install
```

## Building

### Development Build

```bash
# Build WASM module and start dev server
npm run dev
```

### Production Build

```bash
# Build everything for production
npm run build

# Serve built files with proper WASM MIME types
npm run serve

# Then open http://localhost:3000
# Test WASM loading at http://localhost:3000/test.html
```

## 🔧 **WASM Loading Fixes**

The application includes solutions for common WASM serving issues:

- **MIME Type**: Custom Python server sets `application/wasm` MIME type
- **CORS Headers**: Proper headers for WASM in modern browsers
- **Static Serving**: WASM files served from `/` root path
- **Dynamic Loading**: Runtime WASM import with explicit paths

### Individual Components

```bash
# Build only WASM (development)
npm run build-wasm

# Build only WASM (release)
npm run build-wasm-release

# Build only TypeScript
npm run build-ts

# Type check without building
npm run type-check
```

## Architecture

### Backend (Rust WASM)

- `createBoard()`: Generate a completely solved 9×9 board
- `createGame(difficulty)`: Create puzzle by removing cells
- `validateBoard(board)`: Check for conflicts and completion
- `solveBoard(board)`: Solve any valid puzzle

### Frontend (TypeScript)

- **DatabaseManager**: IndexedDB operations for persistence
- **GameManager**: Game state and UI coordination
- **DOM Utilities**: Type-safe element creation helpers

### Persistence Schema

- **games** store: `{ id: string, created: number }`
- **cells** store: `{ gameId: string, cellIndex: number, value: number|null, isGiven: boolean }`

## Game Flow

1. **Start Screen**: Choose "Start New Game" or "Continue Last Game"
2. **New Game**: Generates UUID, creates puzzle, saves to IndexedDB, updates URL
3. **Continue**: Loads most recent game from database, restores board state
4. **Gameplay**: Click cells to edit, automatic validation and persistence
5. **URL Routing**: `?gameId=<uuid>` enables direct game links

## Development Notes

- WASM module exports are globally available as `window.wasm`
- CSS Grid creates 9×9 board with bold lines every 3 cells
- IndexedDB handles offline persistence
- URL parameters enable bookmarkable games
- Input validation restricts cells to 1-9 or empty

## Browser Support

- Modern browsers with WASM, IndexedDB, and CSS Grid support
- ES2022+ features used in TypeScript
- No polyfills included (targets recent browsers)

## License

MIT
