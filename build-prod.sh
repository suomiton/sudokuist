#!/bin/bash

# Production Build Script for Sudokuist
# This script builds the application with full optimizations

set -e  # Exit on any error

echo "🚀 Starting Sudokuist Production Build..."

# Clean previous builds
echo "🧹 Cleaning previous builds..."
npm run clean

# Build WASM with release optimizations
echo "🦀 Building WASM with release optimizations..."
npm run build-wasm-release

# Copy WASM files to src for TypeScript compilation
echo "📁 Copying WASM files..."
cp -r pkg src/

# Run TypeScript compilation and Vite build in production mode
echo "🔨 Building TypeScript and bundling with Vite..."
tsc && NODE_ENV=production vite build --mode production

# Copy additional assets
echo "📋 Copying additional assets..."
mkdir -p dist/styles
cp styles/* dist/styles/ 2>/dev/null || true
cp public/* dist/ 2>/dev/null || true

# Generate build report
echo "📊 Generating build analysis..."
du -sh dist/* | sort -hr > dist/build-size-report.txt

echo "✅ Production build complete!"
echo "📂 Output directory: dist/"
echo "📊 Build size report: dist/build-size-report.txt"

# Optional: Show compressed sizes
if command -v gzip &> /dev/null; then
    echo "📦 Compressed sizes:"
    find dist -name "*.js" -o -name "*.css" | while read file; do
        original_size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)
        gzipped_size=$(gzip -c "$file" | wc -c)
        echo "  $(basename "$file"): ${original_size} bytes → ${gzipped_size} bytes (gzipped)"
    done
fi

echo "🎉 Ready for deployment!"
