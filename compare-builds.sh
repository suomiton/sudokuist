#!/bin/bash

# Build Comparison Script
# Compares development vs production build sizes

echo "🔍 Build Size Comparison"
echo "========================"

# Function to get directory size
get_size() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        du -sh "$1" 2>/dev/null | cut -f1
    else
        # Linux
        du -sh "$1" 2>/dev/null | cut -f1
    fi
}

# Build development version
echo "📦 Building development version..."
npm run build-wasm > /dev/null 2>&1
cp -r pkg src/ > /dev/null 2>&1
vite build --mode development --outDir dist-dev > /dev/null 2>&1

# Get sizes
PROD_SIZE=$(get_size dist)
DEV_SIZE=$(get_size dist-dev)

echo ""
echo "📊 Results:"
echo "  Production:  $PROD_SIZE"
echo "  Development: $DEV_SIZE"

echo ""
echo "📁 Production build contents:"
ls -lah dist/ | grep -v "^total"

echo ""
echo "🗜️  Compression ratios:"
echo "  JavaScript files:"
find dist -name "*.js" | while read file; do
    if [[ "$OSTYPE" == "darwin"* ]]; then
        original=$(stat -f%z "$file")
    else
        original=$(stat -c%s "$file")
    fi
    gzipped=$(gzip -c "$file" | wc -c)
    ratio=$(echo "scale=1; ($original - $gzipped) * 100 / $original" | bc -l 2>/dev/null || echo "N/A")
    echo "    $(basename "$file"): ${ratio}% reduction"
done

echo ""
echo "  CSS files:"
find dist -name "*.css" | while read file; do
    if [[ "$OSTYPE" == "darwin"* ]]; then
        original=$(stat -f%z "$file")
    else
        original=$(stat -c%s "$file")
    fi
    gzipped=$(gzip -c "$file" | wc -c)
    ratio=$(echo "scale=1; ($original - $gzipped) * 100 / $original" | bc -l 2>/dev/null || echo "N/A")
    echo "    $(basename "$file"): ${ratio}% reduction"
done

# Cleanup
rm -rf dist-dev

echo ""
echo "✅ Comparison complete!"
