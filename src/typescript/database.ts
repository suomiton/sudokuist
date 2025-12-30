/**
 * Database Manager - Handles IndexedDB operations for game persistence
 */

import type { GameRecord, CellRecord } from "./types.js";

export class DatabaseManager {
	private db: IDBDatabase | null = null;
	private readonly dbName = "sudoku";
	private readonly version = 8; // Incremented to add lastAccessed index

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
					gamesStore.createIndex("lastAccessed", "lastAccessed", {
						unique: false,
					});
				} else {
					// Add lastAccessed index if it doesn't exist
					const transaction = (event.target as IDBOpenDBRequest).transaction;
					const gamesStore = transaction!.objectStore("games");
					if (!gamesStore.indexNames.contains("lastAccessed")) {
						gamesStore.createIndex("lastAccessed", "lastAccessed", {
							unique: false,
						});
					}
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
	 * Update an existing game record (for timer updates)
	 */
	async updateGame(game: GameRecord): Promise<void> {
		if (!this.db) throw new Error("Database not initialized");

		const transaction = this.db.transaction(["games"], "readwrite");
		const store = transaction.objectStore("games");

		return new Promise((resolve, reject) => {
			const request = store.put(game);
			request.onsuccess = () => resolve();
			request.onerror = () => reject(request.error);
		});
	}

	/**
	 * Get the most recently accessed unfinished game
	 */
	async getLastGame(): Promise<GameRecord | null> {
		if (!this.db) throw new Error("Database not initialized");

		const transaction = this.db.transaction(["games"], "readonly");
		const store = transaction.objectStore("games");

		// First try to get the most recently accessed unfinished game
		try {
			const lastAccessedIndex = store.index("lastAccessed");
			const unfinishedGame = await new Promise<GameRecord | null>(
				(resolve, reject) => {
					const request = lastAccessedIndex.openCursor(null, "prev");
					request.onsuccess = () => {
						const cursor = request.result;
						if (cursor) {
							const game = cursor.value as GameRecord;
							if (!game.isFinished) {
								resolve(game);
								return;
							}
							cursor.continue();
						} else {
							resolve(null);
						}
					};
					request.onerror = () => reject(request.error);
				}
			);

			if (unfinishedGame) {
				return unfinishedGame;
			}
		} catch (error) {
			console.warn(
				"Failed to query by lastAccessed, falling back to created date",
				error
			);
		}

		// Fallback: get most recently created unfinished game
		const createdIndex = store.index("created");
		return new Promise((resolve, reject) => {
			const request = createdIndex.openCursor(null, "prev");
			request.onsuccess = () => {
				const cursor = request.result;
				if (cursor) {
					const game = cursor.value as GameRecord;
					if (!game.isFinished) {
						resolve(game);
						return;
					}
					cursor.continue();
				} else {
					resolve(null);
				}
			};
			request.onerror = () => reject(request.error);
		});
	}

	/**
	 * Get all games ordered by creation date (most recent first)
	 */
	async getAllGames(): Promise<GameRecord[]> {
		if (!this.db) throw new Error("Database not initialized");

		const transaction = this.db.transaction(["games"], "readonly");
		const store = transaction.objectStore("games");
		const index = store.index("created");

		return new Promise((resolve, reject) => {
			const games: GameRecord[] = [];
			const request = index.openCursor(null, "prev");

			request.onsuccess = () => {
				const cursor = request.result;
				if (cursor) {
					games.push(cursor.value);
					cursor.continue();
				} else {
					resolve(games);
				}
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
