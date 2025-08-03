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
	private notesState: number[][] = new Array(81).fill(null).map(() => []); // Notes for each cell
	private givenCells: Set<number> = new Set();
	private currentSeed: number | null = null; // Store the seed used for this puzzle
	private currentDifficulty: number = 1; // Store the current difficulty level
	private db: DatabaseManager;

	// Timer-related properties
	private timerInterval: number | null = null;
	private startTime: number | null = null;
	private elapsedTime: number = 0; // in seconds

	// Hint tracking
	private hintsUsed: number = 0;

	// Numpad properties
	private selectedCellIndex: number | null = null;

	constructor(db: DatabaseManager) {
		this.db = db;
		this.setupNumpad();
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
		this.hintsUsed = 0;

		const gameRecord: GameRecord = {
			id: this.currentGameId,
			created: Date.now(),
			startTime: this.startTime,
			elapsedTime: 0,
			hintsUsed: 0,
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

		// Initialize notes state
		this.notesState = new Array(81).fill(null).map(() => []);

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
		this.hintsUsed = lastGame.hintsUsed || 0;

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
		this.notesState = new Array(81).fill(null).map(() => []);
		this.givenCells.clear();

		cells.forEach((cell) => {
			this.boardState[cell.cellIndex] = cell.value;
			this.notesState[cell.cellIndex] = cell.notes || [];
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
				notes: this.notesState[index],
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

		// Clear notes when setting a value
		if (value !== null) {
			this.notesState[cellIndex] = [];
		}

		// Update the visual cell
		const cell = document.querySelector(
			`[data-index="${cellIndex}"]`
		) as HTMLElement;
		if (cell) {
			if (value !== null) {
				cell.classList.add("user-input");
			} else {
				cell.classList.remove("user-input");
			}
		}

		// Update cell display (including notes)
		this.updateCellDisplay(cellIndex);

		// Update numpad highlighting if this cell is selected (important for clearing note highlights)
		if (this.selectedCellIndex === cellIndex) {
			this.updateNumpadHighlighting(cellIndex);
		}

		if (this.currentGameId) {
			const cellRecord: CellRecord = {
				gameId: this.currentGameId,
				cellIndex,
				value,
				isGiven: false,
				notes: this.notesState[cellIndex],
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

			// Show completion modal with time and hints used
			const timeText = this.formatTime(this.elapsedTime);
			const hintsText =
				this.hintsUsed === 0
					? "without using any hints"
					: this.hintsUsed === 1
					? "using 1 hint"
					: `using ${this.hintsUsed} hints`;

			modal.show({
				title: "🎉 Congratulations!",
				message: `Puzzle completed in <strong>${timeText}</strong> ${hintsText}!<br><br>Well done! You've successfully solved the Sudoku puzzle.`,
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
	 * Provide a hint by filling in one empty cell with the correct value
	 */
	async showHint(): Promise<void> {
		// Convert board state to format expected by WASM (0 for empty cells)
		const wasmBoard = this.boardState.map((val) => val ?? 0);
		const solution = wasm.solve_puzzle(wasmBoard);

		// Find all empty cells that can be filled with a hint
		const emptyCells: number[] = [];
		for (let i = 0; i < 81; i++) {
			if (this.boardState[i] === null && !this.givenCells.has(i)) {
				emptyCells.push(i);
			}
		}

		if (emptyCells.length === 0) {
			// No empty cells to fill
			modal.show({
				title: "No Hints Available",
				message: "The puzzle is already complete!",
				type: "info",
				showCancel: false,
				confirmText: "OK",
			});
			return;
		}

		// Pick a random empty cell to fill
		const randomIndex = Math.floor(Math.random() * emptyCells.length);
		const cellIndex = emptyCells[randomIndex];
		const hintValue = solution[cellIndex];

		// Update the cell with the hint
		this.boardState[cellIndex] = hintValue;
		const cell = document.querySelector(
			`[data-index="${cellIndex}"]`
		) as HTMLElement;
		if (cell) {
			cell.classList.add("user-input", "hint-cell");
		}

		// Clear any notes for this cell since we're setting a value
		this.notesState[cellIndex] = [];

		// Update cell display
		this.updateCellDisplay(cellIndex);

		// Increment hints used counter
		this.hintsUsed++;

		// Save the updated cell and game state
		if (this.currentGameId) {
			const cellRecord: CellRecord = {
				gameId: this.currentGameId,
				cellIndex,
				value: hintValue,
				isGiven: false,
				notes: this.notesState[cellIndex],
			};
			await this.db.saveCell(cellRecord);
			await this.saveHintCount();
		}

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
				if (this.givenCells.has(i)) {
					cell.classList.add("given");
				} else {
					cell.classList.add("user-input");
				}
			}

			// Add click/focus handlers for user input
			this.setupCellInteraction(cell, i);
			boardGrid.appendChild(cell);

			// Update cell display (including notes) AFTER appending to DOM
			this.updateCellDisplay(i);
		}

		// Initial validation
		await this.validateAndUpdateUI();
	}

	/**
	 * Setup interaction handlers for a cell
	 */
	private setupCellInteraction(cell: HTMLElement, index: number): void {
		// All cells should have click handlers for consistency
		cell.addEventListener("click", () => {
			if (this.givenCells.has(index)) {
				// If clicking on a given cell, hide the numpad
				this.hideNumpad();
			} else {
				// If clicking on an editable cell, show numpad
				this.showNumpad(index);
			}
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

		// Hide numpad if visible
		this.hideNumpad();

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
		this.hintsUsed = 0;

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
		this.hintsUsed = 0;

		// Generate puzzle using seed - this will always produce the same puzzle
		const gameBoard = wasm.createGameWithSeed(difficulty, BigInt(seed));
		this.boardState = gameBoard.map((val) => val ?? null);

		// Initialize notes state
		this.notesState = new Array(81).fill(null).map(() => []);

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
		this.hintsUsed = 0;

		const gameRecord: GameRecord = {
			id: this.currentGameId,
			created: Date.now(),
			startTime: this.startTime,
			elapsedTime: 0,
			hintsUsed: 0,
		};

		await this.db.saveGame(gameRecord);

		// Use the provided seed for this puzzle
		this.currentSeed = seed;

		// Generate board using WASM with the provided seed
		const gameBoard = wasm.createGameWithSeed(difficulty, BigInt(seed));
		this.boardState = gameBoard.map((val) => val ?? null);

		// Initialize notes state
		this.notesState = new Array(81).fill(null).map(() => []);

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
	 * Save hint count to database
	 */
	private async saveHintCount(): Promise<void> {
		if (!this.currentGameId) return;

		const lastGame = await this.db.getLastGame();
		if (!lastGame || lastGame.id !== this.currentGameId) return;

		const updatedGame: GameRecord = {
			...lastGame,
			hintsUsed: this.hintsUsed,
		};

		try {
			await this.db.updateGame(updatedGame);
		} catch (error) {
			console.warn("Failed to save hint count:", error);
		}
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
			hintsUsed: this.hintsUsed,
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
			hintsUsed: this.hintsUsed,
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

	/**
	 * Toggle a note number in a cell
	 */
	async toggleNote(cellIndex: number, noteValue: number): Promise<void> {
		if (this.givenCells.has(cellIndex)) return; // Can't modify given cells
		if (noteValue < 1 || noteValue > 9) return; // Invalid note value

		const notes = this.notesState[cellIndex];
		const noteIndex = notes.indexOf(noteValue);

		if (noteIndex === -1) {
			// Add note
			notes.push(noteValue);
			notes.sort(); // Keep notes sorted
		} else {
			// Remove note
			notes.splice(noteIndex, 1);
		}

		// Update visual representation
		this.updateCellDisplay(cellIndex);

		// Update numpad highlighting if this cell is selected
		if (this.selectedCellIndex === cellIndex) {
			this.updateNumpadHighlighting(cellIndex);
		}

		// Save to database if we have a current game
		if (this.currentGameId) {
			const cellRecord: CellRecord = {
				gameId: this.currentGameId,
				cellIndex,
				value: this.boardState[cellIndex],
				isGiven: false,
				notes: this.notesState[cellIndex],
			};
			await this.db.saveCell(cellRecord);
		}
	}

	/**
	 * Clear all notes from a cell
	 */
	async clearNotes(cellIndex: number): Promise<void> {
		if (this.givenCells.has(cellIndex)) return;

		this.notesState[cellIndex] = [];
		this.updateCellDisplay(cellIndex);

		// Update numpad highlighting if this cell is selected
		if (this.selectedCellIndex === cellIndex) {
			this.updateNumpadHighlighting(cellIndex);
		}

		// Save to database if we have a current game
		if (this.currentGameId) {
			const cellRecord: CellRecord = {
				gameId: this.currentGameId,
				cellIndex,
				value: this.boardState[cellIndex],
				isGiven: false,
				notes: [],
			};
			await this.db.saveCell(cellRecord);
		}
	}

	/**
	 * Update the visual display of a cell including notes
	 */
	private updateCellDisplay(cellIndex: number): void {
		const cell = document.querySelector(
			`[data-index="${cellIndex}"]`
		) as HTMLElement;
		if (!cell) return;

		const value = this.boardState[cellIndex];
		const notes = this.notesState[cellIndex] || [];

		// Clear existing content
		cell.innerHTML = "";

		// Set the main cell value (same as original logic)
		if (value !== null) {
			cell.textContent = value.toString();
		}

		// Add notes as an absolute positioned element if there are any
		if (notes.length > 0 && value === null) {
			const notesContainer = document.createElement("div");
			notesContainer.className = "cell-notes-display";
			notesContainer.style.position = "absolute";
			notesContainer.style.top = "2px";
			notesContainer.style.left = "2px";
			notesContainer.style.right = "2px";
			notesContainer.style.fontSize = "0.6em";
			notesContainer.style.display = "flex";
			notesContainer.style.flexWrap = "wrap";
			notesContainer.style.gap = "1px";
			notesContainer.style.pointerEvents = "none";

			notes.forEach((note) => {
				const noteSpan = document.createElement("span");
				noteSpan.textContent = note.toString();
				noteSpan.style.opacity = "0.7";
				notesContainer.appendChild(noteSpan);
			});

			cell.appendChild(notesContainer);
		}
	}

	/**
	 * Update numpad highlighting based on current cell's notes
	 */
	private updateNumpadHighlighting(cellIndex: number): void {
		const numpadButtons = document.querySelectorAll(".numpad-btn[data-value]");
		const notes = this.notesState[cellIndex];

		numpadButtons.forEach((button) => {
			const value = button.getAttribute("data-value");
			if (value && value !== "clear") {
				const numValue = parseInt(value);
				if (numValue >= 1 && numValue <= 9) {
					if (notes.includes(numValue)) {
						button.classList.add("has-note");
					} else {
						button.classList.remove("has-note");
					}
				}
			}
		});
	}

	/**
	 * Setup universal numpad event listeners and functionality
	 */
	private setupNumpad(): void {
		const numpad = document.getElementById("number-picker");
		const numpadButtons = document.querySelectorAll(".numpad-btn");

		if (!numpad) return;

		// Handle numpad button clicks
		numpadButtons.forEach((button) => {
			let longPressTimer: number | null = null;
			let isLongPress = false;

			// Touch start - start long press timer
			button.addEventListener("touchstart", (e) => {
				isLongPress = false;
				longPressTimer = window.setTimeout(() => {
					isLongPress = true;
					// Add haptic feedback for long press
					if ("vibrate" in navigator) {
						navigator.vibrate(100);
					}

					// Handle long press as note toggle
					const target = e.target as HTMLElement;
					const value = target.getAttribute("data-value");

					if (
						this.selectedCellIndex !== null &&
						value &&
						value !== "0" &&
						value !== "clear" &&
						value !== "delete"
					) {
						const numValue = parseInt(value);
						if (numValue >= 1 && numValue <= 9) {
							this.toggleNote(this.selectedCellIndex, numValue);
						}
					}
				}, 500); // 500ms long press threshold
			});

			// Touch end - clear timer and handle regular tap if not long press
			button.addEventListener("touchend", (e) => {
				if (longPressTimer) {
					clearTimeout(longPressTimer);
					longPressTimer = null;
				}

				// If it wasn't a long press, handle as regular tap
				if (!isLongPress) {
					// Add haptic feedback on mobile devices
					if ("vibrate" in navigator) {
						navigator.vibrate(50);
					}

					const target = e.target as HTMLElement;
					const value = target.getAttribute("data-value");

					if (this.selectedCellIndex !== null) {
						if (value === "clear" || value === "delete") {
							// Clear value and notes
							this.updateCell(this.selectedCellIndex, null);
							this.clearNotes(this.selectedCellIndex);
						} else if (value && value !== "0") {
							// Only allow numbers 1-9 for Sudoku
							const numValue = parseInt(value);
							if (numValue >= 1 && numValue <= 9) {
								// Regular tap - set value
								this.updateCell(this.selectedCellIndex, numValue);
							}
						}
					}
				}

				isLongPress = false;
			});

			// Mouse click for desktop - handle with shift key
			button.addEventListener("click", (e) => {
				// Skip if this was a touch event (handled above)
				const mouseEvent = e as MouseEvent;
				if (mouseEvent.detail === 0) return; // Touch events have detail = 0

				// Add haptic feedback on mobile devices
				if ("vibrate" in navigator) {
					navigator.vibrate(50);
				}

				const target = e.target as HTMLElement;
				const value = target.getAttribute("data-value");

				if (this.selectedCellIndex !== null) {
					if (value === "clear" || value === "delete") {
						// Clear value and notes
						this.updateCell(this.selectedCellIndex, null);
						this.clearNotes(this.selectedCellIndex);
					} else if (value && value !== "0") {
						// Only allow numbers 1-9 for Sudoku
						const numValue = parseInt(value);
						if (numValue >= 1 && numValue <= 9) {
							// Check if this is a note input (Shift key on desktop)
							if ((e as MouseEvent).shiftKey) {
								this.toggleNote(this.selectedCellIndex, numValue);
							} else {
								// Regular click - set value
								this.updateCell(this.selectedCellIndex, numValue);
							}
						}
					}
				}

				// Keep numpad open for continuous input
			});
		});

		// Close numpad when clicking outside
		document.addEventListener("click", (e) => {
			const target = e.target as HTMLElement;
			if (!numpad.contains(target) && !target.closest(".cell")) {
				this.hideNumpad();
			}
		});

		// Handle hardware keyboard input even when numpad is visible
		document.addEventListener("keydown", (e) => {
			if (
				this.selectedCellIndex !== null &&
				numpad.classList.contains("show")
			) {
				if (e.key >= "1" && e.key <= "9") {
					e.preventDefault();
					const value = parseInt(e.key);
					if (e.shiftKey) {
						// Shift+number = toggle note
						this.toggleNote(this.selectedCellIndex, value);
					} else {
						// Regular number input
						this.updateCell(this.selectedCellIndex, value);
					}
				} else if (["Backspace", "Delete", " "].includes(e.key)) {
					e.preventDefault();
					if (e.shiftKey) {
						// Shift+delete = clear notes only
						this.clearNotes(this.selectedCellIndex);
					} else {
						// Regular delete = clear value and notes
						this.updateCell(this.selectedCellIndex, null);
						this.clearNotes(this.selectedCellIndex);
					}
				} else if (e.key === "Escape") {
					e.preventDefault();
					this.hideNumpad();
				}
			}
		});
	}

	/**
	 * Show numpad for the selected cell
	 */
	private showNumpad(cellIndex: number): void {
		this.selectedCellIndex = cellIndex;

		const numpad = document.getElementById("number-picker");
		if (!numpad) return;

		// Add class to body to trigger layout adjustments
		document.body.classList.add("numpad-visible");

		// Show numpad with animation
		numpad.classList.remove("hidden");
		requestAnimationFrame(() => {
			numpad.classList.add("show");
		});

		// Update selected cell visual feedback
		this.updateSelectedCellHighlight(cellIndex);

		// Update numpad highlighting based on current cell's notes
		this.updateNumpadHighlighting(cellIndex);
	}

	/**
	 * Hide numpad
	 */
	private hideNumpad(): void {
		const numpad = document.getElementById("number-picker");
		if (!numpad) return;

		// Remove numpad visibility
		numpad.classList.remove("show");

		// Remove layout adjustments
		document.body.classList.remove("numpad-visible");

		// Hide numpad after animation
		setTimeout(() => {
			numpad.classList.add("hidden");
		}, 300);

		// Clear selected cell
		this.selectedCellIndex = null;
		this.clearSelectedCellHighlight();
	}

	/**
	 * Update visual highlight for selected cell
	 */
	private updateSelectedCellHighlight(cellIndex: number): void {
		// Remove previous highlights
		document.querySelectorAll(".cell.selected").forEach((cell) => {
			cell.classList.remove("selected");
		});

		// Add highlight to selected cell
		const cell = document.querySelector(`[data-index="${cellIndex}"]`);
		if (cell) {
			cell.classList.add("selected");
		}
	}

	/**
	 * Clear selected cell highlight
	 */
	private clearSelectedCellHighlight(): void {
		document.querySelectorAll(".cell.selected").forEach((cell) => {
			cell.classList.remove("selected");
		});
	}
}
