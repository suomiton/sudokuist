#!/bin/bash

# Development setup script for Sudokuist

echo "🎯 Setting up Sudokuist development environment..."

# Build WASM module
echo "📦 Building Rust WASM module..."
wasm-pack build --target web --out-dir pkg --dev

# Copy WASM files to public directory for static serving
echo "📂 Copying WASM files to public directory..."
mkdir -p public
cp pkg/sudoku_wasm.js pkg/sudoku_wasm_bg.wasm public/

# Copy WASM files to src for TypeScript to find them during build
echo "📂 Copying WASM files to src directory..."
cp -r pkg src/

# Build TypeScript
echo "🔧 Building TypeScript frontend..."
npm run build-ts

echo "✅ Build complete!"
echo ""
echo "🚀 To start development server, run:"
echo "   npx vite --port 3000"
echo ""
echo "📱 Or serve the built application:"
echo "   npx serve dist -p 3000"
echo ""
echo "🌐 Then open http://localhost:3000 in your browser"
