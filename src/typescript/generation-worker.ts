/// <reference lib="webworker" />

import initWasm, * as wasm from "../pkg/sudoku_wasm.js";

let wasmReady: Promise<void> | null = null;

async function ensureWasm(): Promise<void> {
	if (!wasmReady) {
		wasmReady = initWasm().then(() => {});
	}
	return wasmReady || Promise.resolve();
}

self.onmessage = async (event: MessageEvent) => {
	const { id, type, difficulty, seed } = event.data || {};
	if (type !== "generate") return;

	try {
		await ensureWasm();

		if (typeof wasm.register_progress_callback === "function") {
			wasm.register_progress_callback(
				(progress: number, stage: string, meta: any) => {
					(self as DedicatedWorkerGlobalScope).postMessage({
						id,
						type: "progress",
						progress,
						stage,
						meta,
					});
				}
			);
		}

		const gameBoard = wasm.createGameWithSeed(difficulty, BigInt(seed));

		if (typeof wasm.clear_progress_callback === "function") {
			wasm.clear_progress_callback();
		}

		(self as DedicatedWorkerGlobalScope).postMessage({
			id,
			type: "result",
			board: Array.from(gameBoard),
		});
	} catch (error: unknown) {
		(self as DedicatedWorkerGlobalScope).postMessage({
			id,
			type: "error",
			message:
				error instanceof Error
					? error.message
					: "Unknown error during generation",
		});
	}
};

export {};
