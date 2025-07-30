/**
 * Game Manager - Handles game logic and UI state
 */

import type { GameRecord, CellRecord, ValidationResult } from "./types.js";
import { DatabaseManager } from "./database.js";
import { generateUUID, createElement } from "./utils.js";
import { modal } from "./modal.js";

export class GameManager {
	private currentGameId: string | null = null;
	private boardState: (number | null)[] = new Array(81).fill(null);
	private givenCells: Set<number> = new Set();
	private currentSeed: number | null = null; // Store the seed used for this puzzle
	private currentDifficulty: number = 1; // Store the current difficulty level
	private db: DatabaseManager;

	// Timer-related properties
	private timerInterval: number | null = null;
	private startTime: number | null = null;
	private elapsedTime: number = 0; // in seconds

	constructor(db: DatabaseManager) {
		this.db = db;
	}

	/**
	 * Start a new game with given difficulty
	 */
	async startNewGame(difficulty: number = 1): Promise<void> {
		// Store the difficulty for this game
		this.currentDifficulty = difficulty;

		// Generate new game ID and create game record
		this.currentGameId = generateUUID();
		this.startTime = Date.now();
		this.elapsedTime = 0;

		const gameRecord: GameRecord = {
			id: this.currentGameId,
			created: Date.now(),
			startTime: this.startTime,
			elapsedTime: 0,
		};

		await this.db.saveGame(gameRecord);

		// Generate a random seed for this puzzle
		this.currentSeed = Math.floor(Math.random() * 1000000) + Date.now();

		// Generate board using WASM with seed
		const gameBoard = wasm.createGameWithSeed(
			difficulty,
			BigInt(this.currentSeed)
		);
		this.boardState = gameBoard.map((val) => val ?? null);

		// Track which cells are given (pre-filled)
		this.givenCells.clear();
		this.boardState.forEach((value, index) => {
			if (value !== null) {
				this.givenCells.add(index);
			}
		});

		// Save initial board state to database
		await this.saveBoardState();

		// Update URL and UI
		this.updateURL();
		await this.showGameBoard();
		this.startTimer();
	}

	/**
	 * Continue the most recent game
	 */
	async continueLastGame(): Promise<void> {
		const lastGame = await this.db.getLastGame();
		if (!lastGame) {
			alert("No saved games found");
			return;
		}

		this.currentGameId = lastGame.id;

		// Restore timer state
		this.startTime = lastGame.startTime || Date.now();
		this.elapsedTime = lastGame.elapsedTime || 0;

		await this.loadGameState();

		this.updateURL();
		await this.showGameBoard();
		this.startTimer();
	}

	/**
	 * Load game state from database
	 */
	async loadGameState(): Promise<void> {
		if (!this.currentGameId) return;

		const cells = await this.db.loadCells(this.currentGameId);
		this.boardState = new Array(81).fill(null);
		this.givenCells.clear();

		cells.forEach((cell) => {
			this.boardState[cell.cellIndex] = cell.value;
			if (cell.isGiven) {
				this.givenCells.add(cell.cellIndex);
			}
		});
	}

	/**
	 * Save current board state to database
	 */
	async saveBoardState(): Promise<void> {
		if (!this.currentGameId) return;

		const savePromises = this.boardState.map(async (value, index) => {
			const cellRecord: CellRecord = {
				gameId: this.currentGameId!,
				cellIndex: index,
				value,
				isGiven: this.givenCells.has(index),
			};
			return this.db.saveCell(cellRecord);
		});

		await Promise.all(savePromises);
	}

	/**
	 * Update cell value and persist to database
	 */
	async updateCell(cellIndex: number, value: number | null): Promise<void> {
		if (this.givenCells.has(cellIndex)) return; // Can't modify given cells

		this.boardState[cellIndex] = value;

		if (this.currentGameId) {
			const cellRecord: CellRecord = {
				gameId: this.currentGameId,
				cellIndex,
				value,
				isGiven: false,
			};
			await this.db.saveCell(cellRecord);
		}

		await this.validateAndUpdateUI();
	}

	/**
	 * Validate board and update UI accordingly
	 */
	private async validateAndUpdateUI(): Promise<void> {
		const result = wasm.validateBoard(
			this.boardState.map((val) => val ?? undefined)
		);

		// The result is already a JavaScript object, no need to parse JSON
		const validation: ValidationResult = result as ValidationResult;

		// Clear previous invalid states
		document.querySelectorAll(".cell.invalid").forEach((cell) => {
			cell.classList.remove("invalid");
		});

		// Mark invalid cells
		validation.invalidIndices.forEach((index: number) => {
			const cell = document.querySelector(`[data-index="${index}"]`);
			if (cell) {
				cell.classList.add("invalid");
			}
		});

		// Handle completion
		if (validation.isComplete) {
			// Stop the timer when puzzle is completed
			this.stopTimer();

			// Mark game as finished in database
			await this.markGameAsFinished();

			// Show completion modal with time
			const timeText = this.formatTime(this.elapsedTime);
			modal.show({
				title: "🎉 Congratulations!",
				message: `Puzzle completed in <strong>${timeText}</strong>!<br><br>Well done! You've successfully solved the Sudoku puzzle.`,
				type: "success",
				showCancel: false,
				confirmText: "Back to Menu",
				onConfirm: () => {
					// Return to menu after completion
					this.returnToMenu();
				},
			});
		}
	}

	/**
	 * Show the solution using WASM solver
	 */
	async showSolution(): Promise<void> {
		const solution = wasm.solveBoard(
			this.boardState.map((val) => val ?? undefined)
		);
		Array.from(solution).forEach((value, index) => {
			this.boardState[index] = value;
			const cell = document.querySelector(
				`[data-index="${index}"]`
			) as HTMLElement;
			if (cell) {
				cell.textContent = value.toString();
				if (!this.givenCells.has(index)) {
					cell.classList.add("user-input");
				}
			}
		});
		this.saveBoardState();
		await this.validateAndUpdateUI();
	}

	/**
	 * Create and display the game board UI
	 */
	private async showGameBoard(): Promise<void> {
		const startScreen = document.getElementById("start-screen");
		const sudokuContainer = document.getElementById("sudoku-container");
		const boardGrid = document.getElementById("sudoku-board");

		if (!startScreen || !sudokuContainer || !boardGrid) return;

		// Hide start screen, show game
		startScreen.classList.add("hidden");
		sudokuContainer.classList.remove("hidden");

		// Update share link for the current game
		this.updateShareLink();

		// Clear existing board
		boardGrid.innerHTML = "";

		// Create 81 cells (9x9 grid)
		for (let i = 0; i < 81; i++) {
			const cell = createElement("div", {
				className: "cell",
				tabIndex: 0,
				dataset: { index: i.toString() },
			});

			// Set initial value and styling
			const value = this.boardState[i];
			if (value !== null) {
				cell.textContent = value.toString();
				if (this.givenCells.has(i)) {
					cell.classList.add("given");
				} else {
					cell.classList.add("user-input");
				}
			}

			// Add click/focus handlers for user input
			this.setupCellInteraction(cell, i);
			boardGrid.appendChild(cell);
		}

		// Initial validation
		await this.validateAndUpdateUI();
	}

	/**
	 * Setup interaction handlers for a cell
	 */
	private setupCellInteraction(cell: HTMLElement, index: number): void {
		if (this.givenCells.has(index)) return; // Given cells are not editable

		cell.addEventListener("click", () => {
			cell.contentEditable = "true";
			cell.focus();

			// Select all text for easy replacement
			const range = document.createRange();
			range.selectNodeContents(cell);
			const selection = window.getSelection();
			if (selection) {
				selection.removeAllRanges();
				selection.addRange(range);
			}
		});

		cell.addEventListener("keydown", (e) => {
			// Handle number input (1-9)
			if (e.key >= "1" && e.key <= "9") {
				e.preventDefault();
				const value = parseInt(e.key);
				cell.textContent = value.toString();
				cell.classList.add("user-input");
				this.updateCell(index, value);
				cell.blur();
			}
			// Handle deletion (Backspace, Delete, Space)
			else if (["Backspace", "Delete", " "].includes(e.key)) {
				e.preventDefault();
				cell.textContent = "";
				cell.classList.remove("user-input");
				this.updateCell(index, null);
				cell.blur();
			}
			// Handle escape
			else if (e.key === "Escape") {
				cell.blur();
			}
			// Prevent all other input
			else if (e.key.length === 1) {
				e.preventDefault();
			}
		});

		cell.addEventListener("blur", () => {
			cell.contentEditable = "false";
		});
	}

	/**
	 * Update browser URL with current game ID
	 */
	private updateURL(): void {
		if (this.currentGameId) {
			const url = new URL(window.location.href);
			url.searchParams.set("gameId", this.currentGameId);
			// Remove seed and difficulty parameters when creating a persistent game
			url.searchParams.delete("seed");
			url.searchParams.delete("difficulty");
			window.history.replaceState({}, "", url.toString());
		}
	}

	/**
	 * Return to start screen
	 */
	async returnToMenu(): Promise<void> {
		// Stop the timer
		this.stopTimer();

		const startScreen = document.getElementById("start-screen");
		const sudokuContainer = document.getElementById("sudoku-container");

		if (startScreen && sudokuContainer) {
			sudokuContainer.classList.add("hidden");
			startScreen.classList.remove("hidden");
		}

		// Clear share link
		const shareLink = document.getElementById("share-link") as HTMLInputElement;
		if (shareLink) {
			shareLink.value = "";
		}

		// Clear URL parameters
		const url = new URL(window.location.href);
		url.searchParams.delete("gameId");
		url.searchParams.delete("seed");
		url.searchParams.delete("difficulty");
		window.history.pushState({}, "", url.toString());

		this.currentGameId = null;
		this.currentSeed = null;

		// Update continue button state
		await this.updateContinueButtonState();
	}

	/**
	 * Start a shareable puzzle with given seed and difficulty
	 * This creates an ephemeral puzzle that is not saved to IndexedDB
	 */
	async startShareablePuzzle(seed: number, difficulty: number): Promise<void> {
		// Store the seed for this shareable puzzle
		this.currentSeed = seed;

		// Initialize timer for shared puzzle (non-persistent)
		this.startTime = Date.now();
		this.elapsedTime = 0;

		// Generate puzzle using seed - this will always produce the same puzzle
		const gameBoard = wasm.createGameWithSeed(difficulty, BigInt(seed));
		this.boardState = gameBoard.map((val) => val ?? null);

		// Track which cells are given (pre-filled)
		this.givenCells.clear();
		this.boardState.forEach((value, index) => {
			if (value !== null) {
				this.givenCells.add(index);
			}
		});

		// DO NOT save to IndexedDB - shareable puzzles are ephemeral
		// DO NOT update currentGameId - this is not a persistent game

		// Clear share link for shareable puzzles (they don't have persistent gameId)
		const shareLink = document.getElementById("share-link") as HTMLInputElement;
		if (shareLink) {
			shareLink.value = "";
		}

		// Update URL to include seed and difficulty parameters
		const url = new URL(window.location.href);
		url.searchParams.set("seed", seed.toString());
		url.searchParams.set("difficulty", difficulty.toString());
		url.searchParams.delete("gameId"); // Remove gameId if present
		window.history.replaceState({}, "", url.toString());

		// Show the game board and start timer
		await this.showGameBoard();
		this.startTimer();
	}

	/**
	 * Generate a shareable URL for the current puzzle
	 * This creates a URL with seed and difficulty parameters
	 */
	generateShareableUrl(): string | null {
		// Only generate shareable URL for regular games, not already shared ones
		if (!this.currentGameId) return null;

		// Use the stored seed and difficulty for this puzzle
		let seed = this.currentSeed;
		if (!seed) {
			// Fallback: generate seed from puzzle state if not stored
			seed = this.generateSeedFromPuzzle();
		}

		const url = new URL(window.location.origin + window.location.pathname);
		url.searchParams.set("seed", seed.toString());
		url.searchParams.set("difficulty", this.currentDifficulty.toString());

		return url.toString();
	}

	/**
	 * Generate and populate the share link for the current puzzle
	 */
	private updateShareLink(): void {
		const shareLink = document.getElementById("share-link") as HTMLInputElement;

		if (!shareLink) return;

		// Only show share link for regular games with currentGameId
		if (!this.currentGameId) {
			shareLink.value = "";
			return;
		}

		// Use the stored seed if available, otherwise generate one from puzzle state
		let seed = this.currentSeed;
		if (!seed) {
			// For legacy games without seed, generate a deterministic seed from puzzle
			seed = this.generateSeedFromPuzzle();
		}

		const difficulty = this.currentDifficulty || 1;

		const url = new URL(window.location.origin + window.location.pathname);
		url.searchParams.set("seed", seed.toString());
		url.searchParams.set("difficulty", difficulty.toString());

		shareLink.value = url.toString();
	}

	/**
	 * Generate a deterministic seed from the current puzzle's given cells
	 * This is used for legacy games that don't have a stored seed
	 */
	private generateSeedFromPuzzle(): number {
		// Create a string representation of the given cells (clues)
		let puzzleString = "";
		for (let i = 0; i < 81; i++) {
			if (this.givenCells.has(i)) {
				puzzleString += `${i}:${this.boardState[i]};`;
			}
		}

		// Create a simple hash from the puzzle string
		let hash = 0;
		for (let i = 0; i < puzzleString.length; i++) {
			const char = puzzleString.charCodeAt(i);
			hash = (hash << 5) - hash + char;
			hash = hash & hash; // Convert to 32-bit integer
		}

		// Ensure positive number
		return Math.abs(hash);
	}

	/**
	 * Start a new game from a shared seed and difficulty
	 * This creates a persistent game that can be saved and continued
	 */
	async startNewGameFromSeed(seed: number, difficulty: number): Promise<void> {
		// Store the difficulty for this game
		this.currentDifficulty = difficulty;

		// Generate new game ID and create game record
		this.currentGameId = generateUUID();
		this.startTime = Date.now();
		this.elapsedTime = 0;

		const gameRecord: GameRecord = {
			id: this.currentGameId,
			created: Date.now(),
			startTime: this.startTime,
			elapsedTime: 0,
		};

		await this.db.saveGame(gameRecord);

		// Use the provided seed for this puzzle
		this.currentSeed = seed;

		// Generate board using WASM with the provided seed
		const gameBoard = wasm.createGameWithSeed(difficulty, BigInt(seed));
		this.boardState = gameBoard.map((val) => val ?? null);

		// Track which cells are given (pre-filled)
		this.givenCells.clear();
		this.boardState.forEach((value, index) => {
			if (value !== null) {
				this.givenCells.add(index);
			}
		});

		// Save initial board state to database
		await this.saveBoardState();

		// Update URL to use gameId instead of seed/difficulty
		this.updateURL();
		await this.showGameBoard();
		this.startTimer();
	}

	/**
	 * Start the timer for the current game
	 */
	private startTimer(): void {
		// Clear any existing timer
		this.stopTimer();

		// Update display immediately
		this.updateTimerDisplay();

		// Start interval to update every second
		this.timerInterval = window.setInterval(() => {
			this.elapsedTime++;
			this.updateTimerDisplay();
			this.saveTimerState();
		}, 1000);
	}

	/**
	 * Stop the timer
	 */
	private stopTimer(): void {
		if (this.timerInterval !== null) {
			clearInterval(this.timerInterval);
			this.timerInterval = null;
		}
	}

	/**
	 * Update the timer display in the UI
	 */
	private updateTimerDisplay(): void {
		const timerDisplay = document.getElementById("timer-display");
		if (!timerDisplay) return;

		// Format elapsed time as HH:MM:SS
		const totalSeconds = this.elapsedTime;
		const hours = Math.floor(totalSeconds / 3600);
		const minutes = Math.floor((totalSeconds % 3600) / 60);
		const seconds = totalSeconds % 60;

		// Format with leading zeros
		const formatted = `${hours.toString().padStart(2, "0")}:${minutes
			.toString()
			.padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;

		timerDisplay.textContent = formatted;
	}

	/**
	 * Format time in seconds to HH:MM:SS string
	 */
	private formatTime(totalSeconds: number): string {
		const hours = Math.floor(totalSeconds / 3600);
		const minutes = Math.floor((totalSeconds % 3600) / 60);
		const seconds = totalSeconds % 60;

		// Format with leading zeros
		return `${hours.toString().padStart(2, "0")}:${minutes
			.toString()
			.padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
	}

	/**
	 * Save timer state to database
	 */
	private async saveTimerState(): Promise<void> {
		// Only save timer state for persistent games (not shareable puzzles)
		if (!this.currentGameId || !this.startTime) return;

		const lastGame = await this.db.getLastGame();
		if (!lastGame || lastGame.id !== this.currentGameId) return;

		const updatedGame: GameRecord = {
			...lastGame,
			startTime: this.startTime,
			elapsedTime: this.elapsedTime,
		};

		try {
			await this.db.updateGame(updatedGame);
		} catch (error) {
			console.warn("Failed to save timer state:", error);
		}
	}

	/**
	 * Mark the current game as finished in the database
	 */
	private async markGameAsFinished(): Promise<void> {
		if (!this.currentGameId) return;

		const lastGame = await this.db.getLastGame();
		if (!lastGame || lastGame.id !== this.currentGameId) return;

		const updatedGame: GameRecord = {
			...lastGame,
			elapsedTime: this.elapsedTime,
			isFinished: true,
		};

		try {
			await this.db.updateGame(updatedGame);
		} catch (error) {
			console.warn("Failed to mark game as finished:", error);
		}
	}

	/**
	 * Update the continue button state based on whether there's an unfinished game
	 */
	async updateContinueButtonState(): Promise<void> {
		const btnContinue = document.getElementById(
			"btn-continue"
		) as HTMLButtonElement;
		if (!btnContinue) return;

		const lastGame = await this.db.getLastGame();

		// Show continue button only if there's an unfinished game
		if (lastGame && !lastGame.isFinished) {
			btnContinue.style.display = "";
			btnContinue.disabled = false;
		} else {
			btnContinue.style.display = "none";
		}
	}
}
