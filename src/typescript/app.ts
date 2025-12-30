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
import { getShareParams } from "./utils.js";
import { initializeWasm } from "./wasm-loader.js";
import { modal } from "./modal.js";

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

		// Make gameManager available globally for onclick handlers
		(window as any).gameManager = gameManager;

		// Debug: Make test function available in development
		if (import.meta.env.DEV) {
			(window as any).createTestGames = () => gameManager.createTestGames();
		}

		// Setup event handlers
		await setupEventHandlers(gameManager);

		// Update button states on initialization
		await gameManager.updateContinueButtonState();
		await gameManager.updateScoreboardButtonState();

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
 * Setup all event handlers for the application
 */
async function setupEventHandlers(gameManager: GameManager): Promise<void> {
	const btnStart = document.getElementById("btn-start");
	const btnContinue = document.getElementById("btn-continue");
	const btnScoreboard = document.getElementById("btn-scoreboard");
	const btnBack = document.getElementById("btn-back");
	const btnBackScoreboard = document.getElementById("btn-back-scoreboard");
	const btnSolve = document.getElementById("btn-solve");
	const copyLinkBtn = document.getElementById("copy-link-btn");

	if (btnStart) {
		btnStart.addEventListener("click", async () => {
			const difficulty = await modal.selectDifficulty();
			if (difficulty !== null) {
				gameManager.startNewGame(difficulty);
			}
		});
	}

	if (btnContinue) {
		// Check if there's an unfinished game to continue
		await gameManager.updateContinueButtonState();

		btnContinue.addEventListener("click", () => gameManager.continueLastGame());
	}

	if (btnScoreboard) {
		// Check if there are games to display in scoreboard
		await gameManager.updateScoreboardButtonState();

		btnScoreboard.addEventListener("click", () => gameManager.showScoreboard());
	}

	if (btnBack) {
		btnBack.addEventListener(
			"click",
			async () => await gameManager.returnToMenu()
		);
	}

	if (btnBackScoreboard) {
		btnBackScoreboard.addEventListener(
			"click",
			async () => await gameManager.returnToMenu()
		);
	}

	if (btnSolve) {
		btnSolve.addEventListener("click", async () => {
			const confirmed = await modal.confirm(
				"Use Hint",
				"Do you want to reveal one number? This will count as a hint used."
			);

			if (confirmed) {
				gameManager.showHint();
			}
		});
	}

	if (copyLinkBtn) {
		copyLinkBtn.addEventListener("click", async () => {
			const shareLink = document.getElementById(
				"share-link"
			) as HTMLInputElement;
			if (shareLink) {
				try {
					await navigator.clipboard.writeText(shareLink.value);
					copyLinkBtn.textContent = "Copied!";
					copyLinkBtn.classList.add("copied");

					setTimeout(() => {
						copyLinkBtn.textContent = "Copy";
						copyLinkBtn.classList.remove("copied");
					}, 2000);
				} catch (error) {
					console.error("Failed to copy link:", error);
					// Fallback: select the text
					shareLink.select();
					shareLink.setSelectionRange(0, 99999);
					alert("Link selected. Press Ctrl+C to copy.");
				}
			}
		});
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
