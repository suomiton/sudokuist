import { defineConfig } from 'vite'

export default defineConfig({
	root: '.',
	publicDir: 'public',
	build: {
		outDir: 'dist',
		emptyOutDir: true,
		rollupOptions: {
			input: {
				main: './index.html'
			},
			output: {
				entryFileNames: 'app.js',
				assetFileNames: '[name].[ext]'
			}
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
		exclude: ['../pkg/sudoku_wasm.js']
	},
	assetsInclude: ['**/*.wasm'],
	// Add CSS processing for HMR
	css: {
		devSourcemap: true
	}
})
