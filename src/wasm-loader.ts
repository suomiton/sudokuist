/**
 * WASM module loader and initialization
 */

/**
 * Load WASM module and make it globally available
 */
export async function initializeWasm(): Promise<void> {
	try {
		// Import WASM module - use different paths for dev vs production
		let wasmModule;
		if (import.meta.env.DEV) {
			// During development, import from the copied pkg directory in src
			wasmModule = await import("./pkg/sudoku_wasm.js");
			await wasmModule.default();
		} else {
			// In production, load from the base path (respects Vite's base configuration)
			const baseUrl = (import.meta.env.BASE_URL || "/").replace(/\/$/, "");
			const wasmModulePath = `${baseUrl}/sudoku_wasm.js`;
			const wasmBinaryPath = `${baseUrl}/sudoku_wasm_bg.wasm`;
			wasmModule = await import(/* @vite-ignore */ wasmModulePath);
			await wasmModule.default(wasmBinaryPath);
		}

		// Make WASM functions globally available
		(window as any).wasm = wasmModule;
	} catch (error) {
		console.error("Failed to load WASM module:", error);
		throw new Error(
			"Failed to load game engine. Please make sure the WASM module is built and available."
		);
	}
}
