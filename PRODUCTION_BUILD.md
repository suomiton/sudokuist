# Production Build Guide

This document explains how to build and deploy the Sudokuist application with full production optimizations.

## Quick Start

To create a production build:

```bash
npm run build:prod
```

This will create an optimized build in the `dist/` directory.

## Build Features

### WASM Optimizations

- **Release Mode**: Built with `--release` flag for maximum performance
- **Link Time Optimization (LTO)**: Enables cross-crate optimizations
- **Single Codegen Unit**: Reduces binary size
- **Panic Abort**: Smaller WASM binary size
- **Optimized Dependencies**: All dependencies built with `opt-level = 3`

### JavaScript/TypeScript Optimizations

- **Terser Minification**: Advanced JavaScript minification
- **Dead Code Elimination**: Removes unused code and console logs
- **Code Splitting**: Separate chunks for better caching:
  - `wasm-core`: WASM loading functionality
  - `game-logic`: Game management and database
  - `ui-components`: UI utilities and modal system
- **Content Hashing**: Filenames include content hashes for cache busting
- **Tree Shaking**: Removes unused exports

### CSS Optimizations

- **Minification**: CSS is minified and optimized
- **Source Maps**: Disabled in production for smaller builds

### Build Analysis

The build process includes:

- **Size Report**: Generated in `dist/build-size-report.txt`
- **Gzip Analysis**: Shows compressed sizes for all assets
- **Bundle Analysis**: Use `npm run build:analyze` for detailed analysis

## Build Scripts

### Available Commands

- `npm run build:prod` - Full production build with all optimizations
- `npm run build:analyze` - Build with bundle analysis report
- `npm run build:preview` - Build and start preview server
- `npm run preview` - Preview the built application

### File Sizes (Gzipped)

| Asset Type  | Original | Gzipped | Compression |
| ----------- | -------- | ------- | ----------- |
| WASM Binary | 104KB    | ~30KB   | ~70%        |
| JavaScript  | ~20KB    | ~6KB    | ~70%        |
| CSS         | ~13KB    | ~3KB    | ~77%        |

## Deployment

The production build is completely static and can be deployed to any static hosting service:

### Using Serve (Development)

```bash
npx serve dist -s
```

### Using Serve with Production Config

```bash
npx serve dist -c serve.production.json
```

### Deployment Checklist

1. ✅ WASM files are properly optimized
2. ✅ JavaScript is minified and tree-shaken
3. ✅ CSS is minified
4. ✅ Assets have content hashes for caching
5. ✅ Console logs are stripped in production
6. ✅ Source maps are disabled
7. ✅ Gzip compression is enabled

## Environment Variables

The build process uses the following environment variables:

- `NODE_ENV=production` - Enables production optimizations
- `VITE_BUILD_MODE=production` - Vite production mode

## Performance Notes

- **Initial Load**: ~40KB gzipped (including WASM)
- **Subsequent Loads**: Cached assets, only new chunks downloaded
- **WASM Performance**: Optimized for both size and execution speed
- **Cache Strategy**: Long-term caching with content hashing

## Troubleshooting

### Large Bundle Size

Run `npm run build:analyze` to identify large dependencies.

### WASM Loading Issues

Ensure WASM files are served with correct MIME type (`application/wasm`).

### Cache Issues

Content hashes in filenames ensure proper cache invalidation.

### Build Failures

1. Clean the build: `npm run clean`
2. Reinstall dependencies: `npm install`
3. Check Rust/wasm-pack installation

## Advanced Configuration

The build configuration is in `vite.config.js` and supports:

- Custom Terser options
- Manual chunk splitting
- Asset optimization
- Source map control
- Development vs production builds
