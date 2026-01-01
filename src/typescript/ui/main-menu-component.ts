import { UIComponent, bindEvent } from "./component.js";
import { createElement } from "../utils.js";

export interface MainMenuViewState {
	isVisible: boolean;
	continueButton: { enabled: boolean; title: string };
	scoreboardButton: { enabled: boolean; title: string };
}

export interface MainMenuComponentContext {
	onStart: () => void;
	onContinue: () => void;
	onScoreboard: () => void;
}

export class MainMenuComponent
	implements UIComponent<MainMenuComponentContext, MainMenuViewState>
{
	private container: HTMLElement | null = null;
	private btnStart: HTMLButtonElement | null = null;
	private btnContinue: HTMLButtonElement | null = null;
	private btnScoreboard: HTMLButtonElement | null = null;
	private cleanup: Array<() => void> = [];
	private context: MainMenuComponentContext | null = null;

	/** Store callbacks for menu button actions. */
	init(context: MainMenuComponentContext): void {
		this.context = context;
	}

	/** Build the start screen DOM and attach interaction handlers. */
	mount(container: HTMLElement): void {
		this.unmount();
		this.container = container;
		this.container.innerHTML = "";

		const title = createElement("h1", {}, "Sudokuist");
		const buttonContainer = createElement("div", { className: "button-container" });
		const description = createElement(
			"p",
			{},
			"Welcome to Sudokuist! A traditional Sudoku game designed for puzzle lovers. Play, relax, and sharpen your mind—directly in your browser! Choose an option to begin:"
		);

		this.btnStart = createElement("button", {
			id: "btn-start",
			className: "game-btn",
			textContent: "Start New Game",
		});
		this.btnContinue = createElement("button", {
			id: "btn-continue",
			className: "game-btn",
			textContent: "Continue Last Game",
		});
		this.btnScoreboard = createElement("button", {
			id: "btn-scoreboard",
			className: "game-btn",
			textContent: "Scoreboard",
		});

		buttonContainer.appendChild(description);
		buttonContainer.appendChild(this.btnStart);
		buttonContainer.appendChild(this.btnContinue);
		buttonContainer.appendChild(this.btnScoreboard);

		const footer = createElement("footer");
		const footerText = createElement(
			"p",
			{},
			"Sudoku implementation and game engine by "
		);
		const developer = createElement("div", { className: "developer" }, "Toni Suominen");
		footerText.appendChild(developer);
		footer.appendChild(footerText);

		this.container.appendChild(title);
		this.container.appendChild(buttonContainer);
		this.container.appendChild(footer);

		this.cleanup.push(
			bindEvent(this.btnStart, "click", () => this.context?.onStart()),
			bindEvent(this.btnContinue, "click", () => this.context?.onContinue()),
			bindEvent(this.btnScoreboard, "click", () =>
				this.context?.onScoreboard()
			)
		);
	}

	/** Apply button state updates and toggle menu visibility. */
	update(state: MainMenuViewState): void {
		if (this.container) {
			this.container.classList.toggle("hidden", !state.isVisible);
		}

		if (this.btnContinue) {
			this.btnContinue.disabled = !state.continueButton.enabled;
			this.btnContinue.style.opacity = state.continueButton.enabled ? "1" : "0.5";
			this.btnContinue.title = state.continueButton.title;
		}

		if (this.btnScoreboard) {
			this.btnScoreboard.disabled = !state.scoreboardButton.enabled;
			this.btnScoreboard.style.opacity = state.scoreboardButton.enabled ? "1" : "0.5";
			this.btnScoreboard.title = state.scoreboardButton.title;
		}
	}

	/** Remove event listeners and clear the container. */
	unmount(): void {
		this.cleanup.forEach((cleanup) => cleanup());
		this.cleanup = [];
		this.btnStart = null;
		this.btnContinue = null;
		this.btnScoreboard = null;
		if (this.container) {
			this.container.innerHTML = "";
		}
		this.container = null;
	}
}
