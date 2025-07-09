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
		setupEventHandlers(gameManager);

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
function setupEventHandlers(gameManager: GameManager): void {
	const btnStart = document.getElementById("btn-start");
	const btnContinue = document.getElementById("btn-continue");
	const btnBack = document.getElementById("btn-back");
	const btnSolve = document.getElementById("btn-solve");
	const copyLinkBtn = document.getElementById("copy-link-btn");

	if (btnStart) {
		btnStart.addEventListener("click", () => gameManager.startNewGame(1));
	}

	if (btnContinue) {
		btnContinue.addEventListener("click", () => gameManager.continueLastGame());
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
