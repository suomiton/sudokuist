/**
 * Type definitions for the Sudoku application
 */

export interface GameRecord {
	id: string;
	created: number;
}

export interface CellRecord {
	gameId: string;
	cellIndex: number;
	value: number | null;
	isGiven: boolean;
}

export interface ValidationResult {
	invalid_indices: number[];
	is_complete: boolean;
}

// WASM module interface (will be available after loading)
export interface WasmModule {
	createBoard(): Uint8Array;
	createGame(difficulty: number): (number | undefined)[];
	createGameWithSeed(difficulty: number, seed: bigint): (number | undefined)[];
	validateBoard(board: (number | undefined)[]): ValidationResult;
	solveBoard(board: (number | undefined)[]): Uint8Array;
}

declare global {
	const wasm: WasmModule;
}
