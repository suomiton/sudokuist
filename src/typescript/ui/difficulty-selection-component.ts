import { UIComponent, bindEvent } from "./component.js";
import { createElement } from "../utils.js";

export interface DifficultySelectionViewState {
	isOpen: boolean;
}

export interface DifficultySelectionComponentContext {
	onClose?: () => void;
}

interface DifficultyOption {
	level: number;
	name: string;
	stars: number;
	description: string;
}

export class DifficultySelectionComponent
	implements
		UIComponent<DifficultySelectionComponentContext, DifficultySelectionViewState>
{
	private container: HTMLElement | null = null;
	private overlay: HTMLElement | null = null;
	private modal: HTMLElement | null = null;
	private cleanup: Array<() => void> = [];
	private context: DifficultySelectionComponentContext | null = null;
	private resolveSelection: ((value: number | null) => void) | null = null;
	private selectedDifficulty = 1;

	private readonly difficulties: DifficultyOption[] = [
		{
			level: 1,
			name: "Very Easy",
			stars: 1,
			description: "35-45 clues, straightforward",
		},
		{
			level: 2,
			name: "Easy",
			stars: 2,
			description: "35-45 clues, gentle challenge",
		},
		{
			level: 3,
			name: "Medium",
			stars: 3,
			description: "30-35 clues, moderate difficulty",
		},
		{
			level: 4,
			name: "Hard",
			stars: 4,
			description: "25-30 clues, challenging",
		},
		{
			level: 5,
			name: "Expert",
			stars: 5,
			description: "17-24 clues, expert level",
		},
	];

	/** Store optional lifecycle callbacks for the selection flow. */
	init(context: DifficultySelectionComponentContext): void {
		this.context = context;
	}

	/** Set the host container for the overlay modal. */
	mount(container: HTMLElement): void {
		this.unmount();
		this.container = container;
	}

	/** Open or close the modal based on the view model state. */
	update(state: DifficultySelectionViewState): void {
		if (state.isOpen) {
			this.showModal();
		} else {
			this.closeModal();
		}
	}

	/** Tear down the overlay and cleanup any outstanding handlers. */
	unmount(): void {
		this.closeModal();
		this.cleanup.forEach((cleanup) => cleanup());
		this.cleanup = [];
		this.container = null;
		this.context = null;
		this.resolveSelection = null;
	}

	/** Present the difficulty selector and resolve when the user makes a choice. */
	requestSelection(): Promise<number | null> {
		return new Promise((resolve) => {
			this.resolveSelection = resolve;
			this.selectedDifficulty = 1;
			this.update({ isOpen: true });
		});
	}

	/** Build and display the difficulty selection modal. */
	private showModal(): void {
		if (!this.container) return;
		if (this.overlay) return;

		this.overlay = createElement("div", { className: "modal-overlay" });
		this.modal = createElement("div", {
			className: "modal-container modal-info",
		});

		const content = createElement("div", { className: "modal-content" });
		const header = createElement("div", { className: "modal-header" });
		header.appendChild(
			createElement("h3", { className: "modal-title" }, "Select Difficulty")
		);

		const body = createElement("div", { className: "modal-body" });
		body.appendChild(
			createElement(
				"p",
				{ className: "modal-message" },
				"Choose your preferred difficulty level:"
			)
		);

		const difficultyContainer = createElement("div", {
			className: "difficulty-selection",
		});

		this.difficulties.forEach((diff) => {
			const option = this.buildOption(diff, difficultyContainer);
			difficultyContainer.appendChild(option);
		});

		body.appendChild(difficultyContainer);

		const footer = createElement("div", { className: "modal-footer" });
		const startButton = createElement("button", {
			className: "modal-btn modal-btn-confirm",
			textContent: "Start Game",
		});
		const cancelButton = createElement("button", {
			className: "modal-btn modal-btn-cancel",
			textContent: "Cancel",
		});

		footer.appendChild(startButton);
		footer.appendChild(cancelButton);

		content.appendChild(header);
		content.appendChild(body);
		content.appendChild(footer);
		this.modal.appendChild(content);
		this.overlay.appendChild(this.modal);
		this.container.appendChild(this.overlay);

		this.cleanup.push(
			bindEvent(startButton, "click", () => this.resolveAndClose(this.selectedDifficulty)),
			bindEvent(cancelButton, "click", () => this.resolveAndClose(null)),
			bindEvent(document, "keydown", (event: KeyboardEvent) => {
				if (event.key === "Escape") {
					this.resolveAndClose(null);
				}
			}),
			bindEvent(this.overlay, "click", (event: MouseEvent) => {
				if (event.target === this.overlay) {
					this.resolveAndClose(null);
				}
			})
		);

		setTimeout(() => startButton.focus(), 100);

		requestAnimationFrame(() => {
			this.overlay?.classList.add("modal-overlay-visible");
			this.modal?.classList.add("modal-container-visible");
		});
	}

	/** Build a single difficulty option entry. */
	private buildOption(
		option: DifficultyOption,
		container: HTMLElement
	): HTMLElement {
		const wrapper = createElement("div", { className: "difficulty-option" });
		if (option.level === this.selectedDifficulty) {
			wrapper.classList.add("selected");
		}

		const starsContainer = createElement("div", { className: "difficulty-stars" });
		for (let i = 1; i <= 5; i++) {
			const star = createElement(
				"span",
				{ className: i <= option.stars ? "star filled" : "star empty" },
				"★"
			);
			starsContainer.appendChild(star);
		}

		const info = createElement("div", { className: "difficulty-info" });
		info.appendChild(
			createElement("div", { className: "difficulty-name" }, option.name)
		);
		info.appendChild(
			createElement(
				"div",
				{ className: "difficulty-description" },
				option.description
			)
		);

		wrapper.appendChild(starsContainer);
		wrapper.appendChild(info);

		this.cleanup.push(
			bindEvent(wrapper, "click", () => {
				container
					.querySelectorAll(".difficulty-option")
					.forEach((opt) => opt.classList.remove("selected"));
				wrapper.classList.add("selected");
				this.selectedDifficulty = option.level;
			})
		);

		return wrapper;
	}

	/** Resolve the pending promise and close the modal UI. */
	private resolveAndClose(value: number | null): void {
		const resolver = this.resolveSelection;
		this.resolveSelection = null;
		this.closeModal();
		resolver?.(value);
		this.context?.onClose?.();
	}

	/** Remove the overlay elements and reset state. */
	private closeModal(): void {
		if (!this.overlay) return;

		const overlay = this.overlay;
		const modal = this.modal;

		overlay.classList.remove("modal-overlay-visible");
		modal?.classList.remove("modal-container-visible");

		setTimeout(() => {
			if (overlay.parentNode) {
				overlay.parentNode.removeChild(overlay);
			}
		}, 300);

		this.overlay = null;
		this.modal = null;
		this.cleanup.forEach((cleanup) => cleanup());
		this.cleanup = [];
	}
}
