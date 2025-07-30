import { defineConfig } from 'vite'
import { resolve } from 'path'

export default defineConfig(({ command, mode }) => {
	const isProduction = mode === 'production'

	return {
		root: '.',
		publicDir: 'public',
		build: {
			outDir: 'dist',
			emptyOutDir: true,
			// Production optimizations
			minify: isProduction ? 'terser' : false,
			sourcemap: !isProduction,
			target: 'es2020',
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
				}
			},
			// Terser options for better minification
			terserOptions: isProduction ? {
				compress: {
					drop_console: true,
					drop_debugger: true,
					pure_funcs: ['console.log', 'console.info', 'console.debug']
				},
				mangle: {
					safari10: true
				},
				format: {
					comments: false
				}
			} : undefined,
			// Enable gzip compression analysis
			reportCompressedSize: isProduction,
			// Chunk size warning limit
			chunkSizeWarningLimit: 1000
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
			devSourcemap: !isProduction,
			// CSS minification and optimization
			postcss: isProduction ? {
				plugins: []
			} : undefined
		},
		// Enable experimental features for better tree-shaking
		esbuild: {
			// Remove console logs in production
			drop: isProduction ? ['console', 'debugger'] : []
		}
	}
})
