#!/bin/bash

echo "🚀 Building optimized production bundle..."

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf dist pkg src/pkg public/*.js public/*.wasm

# Build WASM with release optimizations
echo "🦀 Building WASM with release optimizations..."
wasm-pack build --target web --out-dir pkg --release

# Copy WASM files for TypeScript
echo "📁 Copying WASM files for TypeScript..."
cp -r pkg src/

# Copy WASM files to public directory for Vite
echo "📁 Copying WASM files to public directory..."
cp pkg/sudoku_wasm.js pkg/sudoku_wasm_bg.wasm public/

# Type check
echo "📝 Running TypeScript type check..."
NODE_ENV=production npm run type-check

# Build with Vite in production mode
echo "🔨 Building with Vite in production mode..."
NODE_ENV=production npx vite build --mode production

# Copy additional styles
echo "📋 Copying additional styles..."
mkdir -p dist/styles
cp -r styles/* dist/styles/ 2>/dev/null || true

# Verify WASM files are in place
echo "📋 Verifying WASM files..."
ls -la dist/sudoku_wasm* || echo "⚠️  WASM files not found in dist"

# Compress files with gzip
echo "🗜️  Compressing files with gzip..."
find dist -type f \( -name "*.js" -o -name "*.css" -o -name "*.html" -o -name "*.wasm" \) -exec gzip -9 -k {} \;

# Generate build report
echo "📊 Generating build analysis..."
{
    echo "## Optimized Build Size Report"
    echo "\`\`\`"
    du -sh dist/* | sort -hr
    echo "\`\`\`"
    echo ""
    echo "### File Sizes (Original vs Compressed)"
    echo "\`\`\`"
    find dist -name "*.js" -o -name "*.css" -o -name "*.html" -o -name "*.wasm" | while read file; do
        if [ -f "$file" ]; then
            original_size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file")
            if [ -f "$file.gz" ]; then
                gzipped_size=$(stat -f%z "$file.gz" 2>/dev/null || stat -c%s "$file.gz")
                compression_ratio=$(echo "scale=1; 100 - ($gzipped_size * 100 / $original_size)" | bc -l 2>/dev/null || echo "N/A")
                echo "$(basename "$file"): ${original_size} bytes → ${gzipped_size} bytes (${compression_ratio}% reduction)"
            else
                echo "$(basename "$file"): ${original_size} bytes (no compression)"
            fi
        fi
    done
    echo "\`\`\`"
    echo ""
    echo "### Optimization Features Applied"
    echo "\`\`\`"
    echo "✓ JavaScript: Terser minification with aggressive settings"
    echo "✓ CSS: Built-in Vite CSS minification"
    echo "✓ HTML: html-minifier-terser with full optimization"
    echo "✓ WASM JS: Custom minification plugin"
    echo "✓ Tree-shaking: Enabled for dead code elimination"
    echo "✓ Code splitting: Manual chunks for optimal caching"
    echo "✓ Compression: Gzip level 9 for all assets"
    echo "✓ Asset inlining: Small assets (<4KB) inlined as base64"
    echo "\`\`\`"
} > dist/build-report.md

echo "✅ Optimized build complete!"
echo "📁 Output directory: dist/"
echo "📊 Build report: dist/build-report.md"
echo ""
echo "🌐 To preview locally:"
echo "   npm run preview"
echo ""
echo "📈 Build summary:"
find dist -name "*.js" -o -name "*.css" -o -name "*.html" -o -name "*.wasm" | wc -l | xargs echo "   Files:"
du -sh dist | cut -f1 | xargs echo "   Total size:"
