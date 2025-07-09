/**
 * WASM module loader and initialization
 */

import type { WasmModule } from "./types.js";

/**
 * Load WASM module and make it globally available
 */
export async function initializeWasm(): Promise<void> {
	try {
		// Import WASM module - use different paths for dev vs production
		let wasmModule;
		if (import.meta.env.DEV) {
			// During development, import from the pkg directory
			wasmModule = await import("../pkg/sudoku_wasm.js");
			await wasmModule.default();
		} else {
			// In production, load from public directory
			const wasmModulePath = "/sudoku_wasm.js";
			wasmModule = await import(/* @vite-ignore */ wasmModulePath);
			await wasmModule.default("/sudoku_wasm_bg.wasm");
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
