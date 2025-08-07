import { defineConfig } from 'vite'
import { resolve } from 'path'
import terser from '@rollup/plugin-terser'
import { minify } from 'html-minifier-terser'

export default defineConfig(({ command, mode }) => {
	const isProduction = mode === 'production'

	return {
		root: '.',
		publicDir: 'public',
		plugins: isProduction ? [
			// Custom plugin to minify WASM files and HTML
			{
				name: 'minify-assets',
				writeBundle: async (options, bundle) => {
					const fs = await import('fs')
					const path = await import('path')

					// Read and minify the WASM JS file
					const wasmJsPath = path.resolve('pkg/sudoku_wasm.js')
					const distWasmJsPath = path.resolve('dist/sudoku_wasm.js')

					if (fs.existsSync(wasmJsPath)) {
						const wasmJsContent = fs.readFileSync(wasmJsPath, 'utf8')

						// Simple minification for WASM JS
						const minifiedContent = wasmJsContent
							.replace(/\/\*[\s\S]*?\*\//g, '') // Remove block comments
							.replace(/\/\/.*$/gm, '') // Remove line comments
							.replace(/\s+/g, ' ') // Collapse whitespace
							.replace(/;\s+/g, ';') // Remove spaces after semicolons
							.replace(/{\s+/g, '{') // Remove spaces after opening braces
							.replace(/\s+}/g, '}') // Remove spaces before closing braces
							.trim()

						fs.writeFileSync(distWasmJsPath, minifiedContent)
					}

					// Copy WASM binary
					const wasmPath = path.resolve('pkg/sudoku_wasm_bg.wasm')
					const distWasmPath = path.resolve('dist/sudoku_wasm_bg.wasm')

					if (fs.existsSync(wasmPath)) {
						fs.copyFileSync(wasmPath, distWasmPath)
					}

					// Minify HTML files
					const htmlPath = path.resolve('dist/index.html')
					if (fs.existsSync(htmlPath)) {
						const htmlContent = fs.readFileSync(htmlPath, 'utf8')
						const minifiedHtml = await minify(htmlContent, {
							collapseWhitespace: true,
							removeComments: true,
							removeRedundantAttributes: true,
							removeScriptTypeAttributes: true,
							removeStyleLinkTypeAttributes: true,
							useShortDoctype: true,
							minifyCSS: true,
							minifyJS: true,
							removeEmptyAttributes: true,
							removeOptionalTags: true,
							sortAttributes: true,
							sortClassName: true
						})
						fs.writeFileSync(htmlPath, minifiedHtml)
					}
				}
			}
		] : [],
		build: {
			outDir: 'dist',
			emptyOutDir: true,
			// Production optimizations
			minify: isProduction ? 'terser' : false,
			sourcemap: !isProduction,
			target: 'es2020',
			// Enable compression reporting
			reportCompressedSize: isProduction,
			// Chunk size warning limit
			chunkSizeWarningLimit: 1000,
			// Optimize asset handling
			assetsInlineLimit: 4096, // Inline assets smaller than 4kb
			rollupOptions: {
				input: {
					main: './index.html'
				},
				output: {
					// Use content hashing for production caching
					entryFileNames: isProduction ? 'assets/[name]-[hash].js' : 'app.js',
					chunkFileNames: isProduction ? 'assets/[name]-[hash].js' : '[name].js',
					assetFileNames: isProduction ? 'assets/[name]-[hash].[ext]' : '[name].[ext]',
					// Manual chunks for better caching
					manualChunks: isProduction ? {
						'wasm-core': ['./src/wasm-loader.ts'],
						'game-logic': ['./src/game-manager.ts', './src/database.ts'],
						'ui-components': ['./src/modal.ts', './src/utils.ts']
					} : undefined
				},
				plugins: isProduction ? [
					// Additional terser plugin for better JS minification
					terser({
						compress: {
							drop_console: true,
							drop_debugger: true,
							pure_funcs: ['console.log', 'console.info', 'console.debug', 'console.warn'],
							passes: 2, // Multiple passes for better optimization
							unsafe: true,
							unsafe_comps: true,
							unsafe_Function: true,
							unsafe_math: true,
							unsafe_symbols: true,
							unsafe_methods: true,
							unsafe_proto: true,
							unsafe_regexp: true,
							unsafe_undefined: true
						},
						mangle: {
							safari10: true,
							toplevel: true
						},
						format: {
							comments: false,
							ecma: 2020
						},
						ecma: 2020
					})
				] : []
			}
		},
		server: {
			port: 3000,
			open: true,
			fs: {
				allow: ['..']
			},
			// Enable HMR for all file types
			hmr: {
				overlay: true
			}
		},
		optimizeDeps: {
			exclude: ['./src/pkg/sudoku_wasm.js']
		},
		assetsInclude: ['**/*.wasm'],
		// CSS processing
		css: {
			devSourcemap: !isProduction
		},
		// Enable experimental features for better tree-shaking
		esbuild: {
			// Remove console logs in production
			drop: isProduction ? ['console', 'debugger'] : [],
			// Enable top-level await
			target: 'es2022',
			// Minify identifiers
			minifyIdentifiers: isProduction,
			minifySyntax: isProduction,
			minifyWhitespace: isProduction
		}
	}
})
