/**
 * Sudoku Application - TypeScript Frontend
 *
 * Main application entry point that coordinates all modules:
 * - WASM initialization and game logic
 * - Database management for persistence
 * - Game state management and UI
 * - Event handling and user interactions
 */

/// <reference types="vite/client" />

import { DatabaseManager } from "./database.js";
import { GameManager } from "./game-manager.js";
import { formatElapsedTime, getShareParams } from "./utils.js";
import { initializeWasm } from "./wasm-loader.js";
import { UIShell } from "./ui/ui-shell.js";

/**
 * Application initialization and main entry point
 */
async function initializeApp(): Promise<void> {
	try {
		// Initialize database
		const db = new DatabaseManager();
		await db.initialize();

		const boardContainer = document.getElementById("sudoku-board");
		const numpadContainer = document.getElementById("number-picker");
		if (!boardContainer || !numpadContainer) {
			throw new Error("Sudoku UI containers are missing from the page.");
		}

		const startScreen = document.getElementById("start-screen");
		const sudokuContainer = document.getElementById("sudoku-container");
		const scoreboardContainer = document.getElementById("scoreboard-container");
		const shareContainer = document.getElementById("share-container");
		const hintContainer = document.getElementById("hint-container");
		const timerDisplay = document.getElementById("timer-display");
		const backButton = document.getElementById("btn-back");

		if (
			!startScreen ||
			!sudokuContainer ||
			!scoreboardContainer ||
			!shareContainer ||
			!hintContainer ||
			!timerDisplay
		) {
			throw new Error("Sudoku UI containers are missing from the page.");
		}

		const uiShell = new UIShell({
			startScreen,
			sudokuContainer,
			scoreboardContainer,
			board: boardContainer,
			numpad: numpadContainer,
			shareContainer,
			hintContainer,
			timerDisplay,
			backButton,
		});

		// Initialize game manager
		const gameManager = new GameManager(db, uiShell);

		// Make gameManager available globally for onclick handlers
		(window as any).gameManager = gameManager;

		// Debug: Make test function available in development
		if (import.meta.env.DEV) {
			(window as any).createTestGames = () => gameManager.createTestGames();
		}

		// Setup UI components
		uiShell.initialize({
			menu: {
				onStart: async () => {
					const difficulty = await uiShell.requestDifficultySelection();
					if (difficulty !== null) {
						gameManager.startNewGame(difficulty);
					}
				},
				onContinue: () => gameManager.continueLastGame(),
				onScoreboard: () => gameManager.showScoreboard(),
			},
			scoreboard: {
				onBack: () => gameManager.returnToMenu(),
				onContinue: (gameId) => gameManager.continueGame(gameId),
				onTryAgain: (seed, difficulty) =>
					gameManager.playAgain(seed, difficulty),
				onTryAgainDifficulty: (difficulty) =>
					gameManager.playAgainDifficulty(difficulty),
				formatElapsedTime,
			},
			board: gameManager.getBoardContext(),
			numpad: gameManager.getNumpadContext(),
			hint: {
				onConfirmHint: () => gameManager.showHint(),
			},
			onBackToMenu: () => gameManager.returnToMenu(),
		});

		// Update button states on initialization
		await gameManager.updateMenuState(true);

		// Handle URL routing
		await handleUrlRouting(gameManager, db);

		console.log("Sudoku application initialized successfully");
	} catch (error) {
		console.error("Failed to initialize application:", error);
		alert(
			"Failed to initialize the game. Please refresh the page and try again."
		);
	}
}

/**
 * Handle URL routing for shared puzzles and existing games
 */
async function handleUrlRouting(
	gameManager: GameManager,
	db: DatabaseManager
): Promise<void> {
	// Check for shareable puzzle parameters first (seed + difficulty)
	const shareParams = getShareParams();
	if (shareParams) {
		// Create a new persistent game from the shared seed
		await gameManager.startNewGameFromSeed(
			shareParams.seed,
			shareParams.difficulty
		);
		return; // Skip gameId flow
	}

	// Check URL for existing game ID (original flow)
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
}

/**
 * Load WASM module and initialize application when DOM is ready
 */
document.addEventListener("DOMContentLoaded", async () => {
	try {
		// Initialize WASM module
		await initializeWasm();

		// Initialize the application
		await initializeApp();
	} catch (error) {
		console.error("Application initialization failed:", error);
		alert(error instanceof Error ? error.message : "Unknown error occurred");
	}
});
