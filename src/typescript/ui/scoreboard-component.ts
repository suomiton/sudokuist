import type { GameRecord } from "../types.js";
import { UIComponent, bindEvent } from "./component.js";
import { createElement } from "../utils.js";

export interface ScoreboardViewState {
	isVisible: boolean;
	games: GameRecord[];
	errorMessage?: string | null;
}

export interface ScoreboardComponentContext {
	onBack: () => void;
	onContinue: (gameId: string) => void;
	onTryAgain: (seed: number, difficulty: number) => void;
	onTryAgainDifficulty: (difficulty: number) => void;
	formatElapsedTime: (seconds: number) => string;
}

export class ScoreboardComponent
	implements UIComponent<ScoreboardComponentContext, ScoreboardViewState>
{
	private container: HTMLElement | null = null;
	private content: HTMLElement | null = null;
	private cleanup: Array<() => void> = [];
	private rowCleanup: Array<() => void> = [];
	private context: ScoreboardComponentContext | null = null;
	private state: ScoreboardViewState | null = null;

	/** Store callbacks and helpers for scoreboard interactions. */
	init(context: ScoreboardComponentContext): void {
		this.context = context;
	}

	/** Build the scoreboard wrapper DOM and connect the back action. */
	mount(container: HTMLElement): void {
		this.unmount();
		this.container = container;
		this.container.innerHTML = "";

		const header = createElement("div", { className: "scoreboard-header" });
		const title = createElement("h2", {}, "Scoreboard");
		const backButton = createElement("button", {
			id: "btn-back-scoreboard",
			textContent: "← Back to Menu",
		});

		header.appendChild(title);
		header.appendChild(backButton);

		this.content = createElement("div", { id: "scoreboard-content" });

		this.container.appendChild(header);
		this.container.appendChild(this.content);

		this.cleanup.push(bindEvent(backButton, "click", () => this.context?.onBack()));
	}

	/** Render the scoreboard table, empty state, or error state. */
	update(state: ScoreboardViewState): void {
		this.state = state;
		if (this.container) {
			this.container.classList.toggle("hidden", !state.isVisible);
		}
		this.renderContent(state);
	}

	/** Clear DOM and listeners for the scoreboard view. */
	unmount(): void {
		this.cleanup.forEach((cleanup) => cleanup());
		this.cleanup = [];
		this.rowCleanup.forEach((cleanup) => cleanup());
		this.rowCleanup = [];
		if (this.container) {
			this.container.innerHTML = "";
		}
		this.container = null;
		this.content = null;
		this.state = null;
	}

	/** Build content based on the current scoreboard view state. */
	private renderContent(state: ScoreboardViewState): void {
		if (!this.content) return;

		this.rowCleanup.forEach((cleanup) => cleanup());
		this.rowCleanup = [];
		this.content.innerHTML = "";

		if (state.errorMessage) {
			this.content.appendChild(
				this.buildEmptyState("Error Loading Games", state.errorMessage)
			);
			return;
		}

		if (state.games.length === 0) {
			this.content.appendChild(
				this.buildEmptyState(
					"No Games Yet",
					"Start playing to see your game history here!"
				)
			);
			return;
		}

		this.content.appendChild(this.buildTable(state.games));
	}

	/** Create the empty-state messaging block. */
	private buildEmptyState(title: string, message: string): HTMLElement {
		const wrapper = createElement("div", { className: "scoreboard-empty" });
		wrapper.appendChild(createElement("h3", {}, title));
		wrapper.appendChild(createElement("p", {}, message));
		return wrapper;
	}

	/** Assemble the scoreboard table including header and rows. */
	private buildTable(games: GameRecord[]): HTMLElement {
		const tableContainer = createElement("div", { className: "scoreboard-table" });
		const table = createElement("table");

		const thead = createElement("thead");
		thead.innerHTML = `
			<tr>
				<th>Difficulty</th>
				<th>Status</th>
				<th>Started</th>
				<th>Time Played</th>
				<th>Hints</th>
				<th>Action</th>
			</tr>
		`;

		const tbody = createElement("tbody");
		for (const game of games) {
			tbody.appendChild(this.buildRow(game));
		}

		table.appendChild(thead);
		table.appendChild(tbody);
		tableContainer.appendChild(table);
		return tableContainer;
	}

	/** Create a scoreboard row for a single game entry. */
	private buildRow(game: GameRecord): HTMLTableRowElement {
		const row = createElement("tr") as HTMLTableRowElement;
		const difficulty = game.difficulty || 1;
		const stars = this.buildStars(difficulty);

		const startDate = new Date(game.created);
		const userLocale = navigator.language || "en-US";
		const dateStr = startDate.toLocaleDateString(userLocale, {
			year: "numeric",
			month: "short",
			day: "numeric",
		});
		const timeStr = startDate.toLocaleTimeString(userLocale, {
			hour: "2-digit",
			minute: "2-digit",
		});

		const elapsedTimeStr = this.context?.formatElapsedTime(game.elapsedTime || 0) ?? "";
		const isFinished = game.isFinished || false;
		const statusText = isFinished ? "Completed" : "Ongoing";
		const statusClass = isFinished ? "completed" : "ongoing";
		const hintsUsed = game.hintsUsed || 0;

		const difficultyCell = createElement("td");
		const difficultyWrapper = createElement("div", {
			className: "difficulty-cell",
		});
		const difficultyStars = createElement("div", {
			className: "difficulty-stars",
		});
		difficultyStars.appendChild(stars);
		difficultyWrapper.appendChild(difficultyStars);
		difficultyCell.appendChild(difficultyWrapper);

		const statusCell = createElement("td");
		const statusBadge = createElement(
			"span",
			{ className: `status-badge ${statusClass}` },
			statusText
		);
		statusCell.appendChild(statusBadge);

		const startedCell = createElement("td");
		startedCell.innerHTML = `${dateStr}<br><small>${timeStr}</small>`;

		const timeCell = createElement("td", {}, elapsedTimeStr);
		const hintsCell = createElement("td", {}, hintsUsed.toString());
		const actionCell = createElement("td", { className: "action-cell" });
		const actionButton = this.buildActionButton(game);

		actionCell.appendChild(actionButton);

		row.appendChild(difficultyCell);
		row.appendChild(statusCell);
		row.appendChild(startedCell);
		row.appendChild(timeCell);
		row.appendChild(hintsCell);
		row.appendChild(actionCell);

		return row;
	}

	/** Build the difficulty star cluster for a given rating. */
	private buildStars(difficulty: number): DocumentFragment {
		const fragment = document.createDocumentFragment();
		for (let i = 1; i <= 5; i++) {
			const star = createElement(
				"span",
				{ className: i <= difficulty ? "star filled" : "star empty" },
				"★"
			);
			fragment.appendChild(star);
		}
		return fragment;
	}

	/** Create the action button for a scoreboard row and bind its callback. */
	private buildActionButton(game: GameRecord): HTMLButtonElement {
		const isFinished = game.isFinished || false;
		const button = createElement("button", {
			className: `game-action-btn ${isFinished ? "try-again" : "continue"}`,
		}) as HTMLButtonElement;

		if (isFinished) {
			button.textContent = "Try Again";
			if (game.seed && game.difficulty) {
				this.rowCleanup.push(
					bindEvent(button, "click", () =>
						this.context?.onTryAgain(game.seed!, game.difficulty!)
					)
				);
			} else {
				const difficulty = game.difficulty || 1;
				this.rowCleanup.push(
					bindEvent(button, "click", () =>
						this.context?.onTryAgainDifficulty(difficulty)
					)
				);
			}
		} else {
			button.textContent = "Continue";
			this.rowCleanup.push(
				bindEvent(button, "click", () =>
					this.context?.onContinue(game.id)
				)
			);
		}

		return button;
	}
}
