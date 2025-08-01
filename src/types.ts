/**
 * Type definitions for the Sudoku application
 */

export interface GameRecord {
	id: string;
	created: number;
	startTime?: number; // When the timer started (timestamp)
	elapsedTime?: number; // Total elapsed time in seconds
	isFinished?: boolean; // Whether the puzzle has been completed
	hintsUsed?: number; // Number of hints used in this game
}

export interface CellRecord {
	gameId: string;
	cellIndex: number;
	value: number | null;
	isGiven: boolean;
}

export interface ValidationResult {
	invalidIndices: number[];
	isComplete: boolean;
}

// WASM module interface (will be available after loading)
export interface WasmModule {
	createBoard(): Uint8Array;
	createGame(difficulty: number): (number | undefined)[];
	createGameWithSeed(difficulty: number, seed: bigint): (number | undefined)[];
	validateBoard(board: (number | undefined)[]): ValidationResult;
	solve_puzzle(board: number[]): number[];
}

declare global {
	const wasm: WasmModule;
}
