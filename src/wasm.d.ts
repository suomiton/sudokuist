// Type declarations for WASM module
interface ValidationResult {
	invalid_indices: number[];
	is_complete: boolean;
}

declare module "/assets/sudoku_wasm.js" {
	export default function init(
		module_or_path?: string | URL | Request | BufferSource | WebAssembly.Module
	): Promise<void>;
	export function createBoard(): Uint8Array;
	export function createGame(difficulty: number): any;
	export function createGameWithSeed(difficulty: number, seed: bigint): any;
	export function validateBoard(board: any): ValidationResult;
	export function solveBoard(board: any): Uint8Array;
}

declare module "./pkg/sudoku_wasm.js" {
	export default function init(): Promise<void>;
	export function createBoard(): Uint8Array;
	export function createGame(difficulty: number): any;
	export function validateBoard(board: any): ValidationResult;
	export function solveBoard(board: any): Uint8Array;
}

declare module "../pkg/sudoku_wasm.js" {
	export default function init(): Promise<void>;
	export function createBoard(): Uint8Array;
	export function createGame(difficulty: number): any;
	export function validateBoard(board: any): any;
	export function solveBoard(board: any): Uint8Array;
}
