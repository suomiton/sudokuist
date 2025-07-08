/**
 * Sudoku Application - TypeScript Frontend
 *
 * This module handles:
 * - DOM manipulation and event handling
 * - IndexedDB persistence for games and cell states
 * - WASM bridge for Sudoku logic
 * - Game state management and URL routing
 */

// Type definitions for our application
interface GameRecord {
	id: string;
	created: number;
}

interface CellRecord {
	gameId: string;
	cellIndex: number;
	value: number | null;
	isGiven: boolean;
}

interface ValidationResult {
	invalid_indices: number[];
	is_complete: boolean;
}

// WASM module interface (will be available after loading)
declare const wasm: {
	createBoard(): Uint8Array;
	createGame(difficulty: number): (number | undefined)[];
	validateBoard(board: (number | undefined)[]): ValidationResult;
	solveBoard(board: (number | undefined)[]): Uint8Array;
};

/**
 * Utility function to create DOM elements with props and children
 * Provides type safety for HTML element creation
 */
export function createElement<K extends keyof HTMLElementTagNameMap>(
	tag: K,
	props?: Partial<HTMLElementTagNameMap[K]>,
	...children: (Node | string)[]
): HTMLElementTagNameMap[K] {
	const element = document.createElement(tag);

	if (props) {
		// Handle dataset separately since it's read-only
		const { dataset, ...otherProps } = props as any;
		
		// Assign other properties
		Object.assign(element, otherProps);
		
		// Handle dataset manually
		if (dataset) {
			for (const [key, value] of Object.entries(dataset)) {
				element.dataset[key] = value as string;
			}
		}
	}

	children.forEach((child) => {
		if (typeof child === "string") {
			element.appendChild(document.createTextNode(child));
		} else {
			element.appendChild(child);
		}
	});

	return element;
}

/**
 * Generate a UUID v4 for game identification
 */
function generateUUID(): string {
	return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, function (c) {
		const r = (Math.random() * 16) | 0;
		const v = c === "x" ? r : (r & 0x3) | 0x8;
		return v.toString(16);
	});
}

/**
 * Database Manager - Handles IndexedDB operations for game persistence
 */
class DatabaseManager {
	private db: IDBDatabase | null = null;
	private readonly dbName = "sudoku";
	private readonly version = 1;

	/**
	 * Initialize IndexedDB with required object stores
	 */
	async initialize(): Promise<void> {
		return new Promise((resolve, reject) => {
			const request = indexedDB.open(this.dbName, this.version);

			request.onerror = () => reject(request.error);
			request.onsuccess = () => {
				this.db = request.result;
				resolve();
			};

			request.onupgradeneeded = (event) => {
				const db = (event.target as IDBOpenDBRequest).result;

				// Games store: tracks game sessions
				if (!db.objectStoreNames.contains("games")) {
					const gamesStore = db.createObjectStore("games", { keyPath: "id" });
					gamesStore.createIndex("created", "created", { unique: false });
				}

				// Cells store: tracks individual cell values per game
				if (!db.objectStoreNames.contains("cells")) {
					const cellsStore = db.createObjectStore("cells", {
						keyPath: ["gameId", "cellIndex"],
					});
					cellsStore.createIndex("gameId", "gameId", { unique: false });
				}
			};
		});
	}

	/**
	 * Save a new game record
	 */
	async saveGame(game: GameRecord): Promise<void> {
		if (!this.db) throw new Error("Database not initialized");

		const transaction = this.db.transaction(["games"], "readwrite");
		const store = transaction.objectStore("games");

		return new Promise((resolve, reject) => {
			const request = store.add(game);
			request.onsuccess = () => resolve();
			request.onerror = () => reject(request.error);
		});
	}

	/**
	 * Get the most recently created game
	 */
	async getLastGame(): Promise<GameRecord | null> {
		if (!this.db) throw new Error("Database not initialized");

		const transaction = this.db.transaction(["games"], "readonly");
		const store = transaction.objectStore("games");
		const index = store.index("created");

		return new Promise((resolve, reject) => {
			const request = index.openCursor(null, "prev");
			request.onsuccess = () => {
				const cursor = request.result;
				resolve(cursor ? cursor.value : null);
			};
			request.onerror = () => reject(request.error);
		});
	}

	/**
	 * Save cell state for a specific game
	 */
	async saveCell(cell: CellRecord): Promise<void> {
		if (!this.db) throw new Error("Database not initialized");

		const transaction = this.db.transaction(["cells"], "readwrite");
		const store = transaction.objectStore("cells");

		return new Promise((resolve, reject) => {
			const request = store.put(cell);
			request.onsuccess = () => resolve();
			request.onerror = () => reject(request.error);
		});
	}

	/**
	 * Load all cells for a specific game
	 */
	async loadCells(gameId: string): Promise<CellRecord[]> {
		if (!this.db) throw new Error("Database not initialized");

		const transaction = this.db.transaction(["cells"], "readonly");
		const store = transaction.objectStore("cells");
		const index = store.index("gameId");

		return new Promise((resolve, reject) => {
			const request = index.getAll(gameId);
			request.onsuccess = () => resolve(request.result);
			request.onerror = () => reject(request.error);
		});
	}
}

/**
 * Game Manager - Handles game logic and UI state
 */
class GameManager {
	private currentGameId: string | null = null;
	private boardState: (number | null)[] = new Array(81).fill(null);
	private givenCells: Set<number> = new Set();
	private db: DatabaseManager;

	constructor(db: DatabaseManager) {
		this.db = db;
	}

	/**
	 * Start a new game with given difficulty
	 */
	async startNewGame(difficulty: number = 1): Promise<void> {
		// Generate new game ID and create game record
		this.currentGameId = generateUUID();
		const gameRecord: GameRecord = {
			id: this.currentGameId,
			created: Date.now(),
		};

		await this.db.saveGame(gameRecord);

		// Generate board using WASM
		const gameBoard = wasm.createGame(difficulty);
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
		this.showGameBoard();
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
		await this.loadGameState();
		this.updateURL();
		this.showGameBoard();
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

		this.validateAndUpdateUI();
	}

	/**
	 * Validate board and update UI accordingly
	 */
	private validateAndUpdateUI(): void {
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
		validation.invalid_indices.forEach((index: number) => {
			const cell = document.querySelector(`[data-index="${index}"]`);
			if (cell) {
				cell.classList.add("invalid");
			}
		});

		// Update status
		const statusText = document.getElementById("status-text");
		if (statusText) {
			if (validation.is_complete) {
				statusText.textContent = "🎉 Congratulations! Puzzle completed!";
				statusText.className = "status-complete";
			} else if (validation.invalid_indices.length > 0) {
				statusText.textContent = "❌ Invalid numbers detected";
				statusText.className = "status-invalid";
			} else {
				statusText.textContent = "Fill in the numbers 1-9";
				statusText.className = "";
			}
		}
	}

	/**
	 * Show the solution using WASM solver
	 */
	showSolution(): void {
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
		this.validateAndUpdateUI();
	}

	/**
	 * Create and display the game board UI
	 */
	private showGameBoard(): void {
		const startScreen = document.getElementById("start-screen");
		const sudokuContainer = document.getElementById("sudoku-container");
		const boardGrid = document.getElementById("sudoku-board");

		if (!startScreen || !sudokuContainer || !boardGrid) return;

		// Hide start screen, show game
		startScreen.classList.add("hidden");
		sudokuContainer.classList.remove("hidden");

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
		this.validateAndUpdateUI();
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
			window.history.pushState({}, "", url.toString());
		}
	}

	/**
	 * Return to start screen
	 */
	returnToMenu(): void {
		const startScreen = document.getElementById("start-screen");
		const sudokuContainer = document.getElementById("sudoku-container");

		if (startScreen && sudokuContainer) {
			sudokuContainer.classList.add("hidden");
			startScreen.classList.remove("hidden");
		}

		// Clear URL parameters
		const url = new URL(window.location.href);
		url.searchParams.delete("gameId");
		window.history.pushState({}, "", url.toString());

		this.currentGameId = null;
	}
}

/**
 * Application initialization and main entry point
 */
async function initializeApp(): Promise<void> {
	try {
		// Initialize database
		const db = new DatabaseManager();
		await db.initialize();

		// Initialize game manager
		const gameManager = new GameManager(db);

		// Setup event handlers
		const btnStart = document.getElementById("btn-start");
		const btnContinue = document.getElementById("btn-continue");
		const btnBack = document.getElementById("btn-back");
		const btnSolve = document.getElementById("btn-solve");

		if (btnStart) {
			btnStart.addEventListener("click", () => gameManager.startNewGame(1));
		}

		if (btnContinue) {
			btnContinue.addEventListener("click", () =>
				gameManager.continueLastGame()
			);
		}

		if (btnBack) {
			btnBack.addEventListener("click", () => gameManager.returnToMenu());
		}

		if (btnSolve) {
			btnSolve.addEventListener("click", () => {
				if (
					confirm(
						"Are you sure you want to see the solution? This will complete the puzzle."
					)
				) {
					gameManager.showSolution();
				}
			});
		}

		// Check URL for existing game ID
		const urlParams = new URLSearchParams(window.location.search);
		const gameId = urlParams.get("gameId");

		if (gameId) {
			// Try to load existing game
			const lastGame = await db.getLastGame();
			if (lastGame && lastGame.id === gameId) {
				await gameManager.continueLastGame();
			} else {
				// Game ID in URL doesn't match any saved game, show start screen
				gameManager.returnToMenu();
			}
		}

		console.log("Sudoku application initialized successfully");
	} catch (error) {
		console.error("Failed to initialize application:", error);
		alert(
			"Failed to initialize the game. Please refresh the page and try again."
		);
	}
}

/**
 * Load WASM module and initialize application when DOM is ready
 */
document.addEventListener("DOMContentLoaded", async () => {
	try {
		// Dynamically import WASM module from public directory
		const wasmModulePath = "/sudoku_wasm.js";
		const wasmModule = await import(/* @vite-ignore */ wasmModulePath);

		// Initialize WASM with explicit path to the .wasm file
		await wasmModule.default("/sudoku_wasm_bg.wasm");

		// Make WASM functions globally available
		(window as any).wasm = wasmModule;

		// Initialize the application
		await initializeApp();
	} catch (error) {
		console.error("Failed to load WASM module:", error);
		alert(
			"Failed to load game engine. Please make sure the WASM module is built and available."
		);
	}
});
