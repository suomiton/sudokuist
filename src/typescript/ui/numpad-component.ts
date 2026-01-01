import { UIComponent, bindEvent } from "./component.js";
import { createElement } from "../utils.js";

export interface NumpadViewState {
	isVisible: boolean;
	notes: number[];
	completedValues: number[];
}

export interface NumpadComponentContext {
	onValueInput: (value: number) => void;
	onNoteToggle: (value: number) => void;
	onClear: (clearNotesOnly: boolean) => void;
	onRequestClose: () => void;
}

export class NumpadComponent
	implements UIComponent<NumpadComponentContext, NumpadViewState>
{
	private container: HTMLElement | null = null;
	private grid: HTMLElement | null = null;
	private buttons: Map<number | "clear", HTMLButtonElement> = new Map();
	private cleanup: Array<() => void> = [];
	private context: NumpadComponentContext | null = null;
	private state: NumpadViewState | null = null;
	private hideTimer: number | null = null;

	init(context: NumpadComponentContext): void {
		this.context = context;
	}

	mount(container: HTMLElement): void {
		this.unmount();
		this.container = container;
		this.container.innerHTML = "";

		this.grid = createElement("div", { className: "numpad-grid" });
		this.container.appendChild(this.grid);

		for (let value = 1; value <= 9; value++) {
			this.createButton(value, value.toString());
		}

		this.createButton("clear", "Clear", ["numpad-clear"]);

		this.cleanup.push(
			bindEvent(document, "click", (event: MouseEvent) => {
				if (!this.state?.isVisible || !this.container) return;
				const target = event.target as HTMLElement;
				if (
					!this.container.contains(target) &&
					!target.closest(".cell")
				) {
					this.context?.onRequestClose();
				}
			}),
			bindEvent(document, "keydown", (event: KeyboardEvent) => {
				if (!this.state?.isVisible) return;

				const numericKey = this.getNumericKey(event);
				if (numericKey !== null) {
					event.preventDefault();
					if (event.shiftKey) {
						this.context?.onNoteToggle(numericKey);
					} else {
						this.context?.onValueInput(numericKey);
					}
					return;
				}

				if (["Backspace", "Delete", " "].includes(event.key)) {
					event.preventDefault();
					this.context?.onClear(event.shiftKey);
					return;
				}

				if (event.key === "Escape") {
					event.preventDefault();
					this.context?.onRequestClose();
				}
			})
		);
	}

	update(state: NumpadViewState): void {
		const wasVisible = this.state?.isVisible ?? false;
		this.state = state;

		this.updateButtonStates(state);

		if (state.isVisible !== wasVisible) {
			this.setVisibility(state.isVisible);
		}
	}

	unmount(): void {
		this.cleanup.forEach((cleanup) => cleanup());
		this.cleanup = [];
		this.buttons.clear();
		if (this.container) {
			this.container.innerHTML = "";
		}
		document.body.classList.remove("numpad-visible");
		this.container = null;
		this.grid = null;
		this.clearHideTimer();
	}

	private createButton(
		value: number | "clear",
		label: string,
		extraClasses: string[] = []
	): void {
		if (!this.grid) return;

		const button = createElement("button", {
			className: ["numpad-btn", ...extraClasses].join(" "),
			dataset: { value: value.toString() },
		}, label);

		let longPressTimer: number | null = null;
		let isLongPress = false;

		this.cleanup.push(
			bindEvent(button, "touchstart", (event: TouchEvent) => {
				if (button.disabled) return;
				isLongPress = false;
				longPressTimer = window.setTimeout(() => {
					isLongPress = true;
					if ("vibrate" in navigator) {
						navigator.vibrate(100);
					}
					this.handleInput(button, true);
				}, 500);
			}),
			bindEvent(button, "touchend", (event: TouchEvent) => {
				if (button.disabled) return;
				if (longPressTimer !== null) {
					clearTimeout(longPressTimer);
					longPressTimer = null;
				}

				if (!isLongPress) {
					if ("vibrate" in navigator) {
						navigator.vibrate(50);
					}
					this.handleInput(button, false);
				}
				isLongPress = false;
			}),
			bindEvent(button, "click", (event: MouseEvent) => {
				if (button.disabled) return;
				if (event.detail === 0) return;

				if ("vibrate" in navigator) {
					navigator.vibrate(50);
				}

				this.handleInput(button, event.shiftKey);
			})
		);

		this.buttons.set(value, button);
		this.grid.appendChild(button);
	}

	private handleInput(
		button: HTMLButtonElement,
		isNote: boolean
	): void {
		const value = button.getAttribute("data-value");
		if (!value || value === "0") return;

		if (value === "clear" || value === "delete") {
			this.context?.onClear(false);
			return;
		}

		const numericValue = parseInt(value, 10);
		if (Number.isInteger(numericValue) && numericValue >= 1 && numericValue <= 9) {
			if (isNote) {
				this.context?.onNoteToggle(numericValue);
			} else {
				this.context?.onValueInput(numericValue);
			}
		}
	}

	private updateButtonStates(state: NumpadViewState): void {
		const completed = new Set(state.completedValues);
		const notes = new Set(state.notes);

		for (let value = 1; value <= 9; value++) {
			const button = this.buttons.get(value);
			if (!button) continue;

			const isComplete = completed.has(value);
			button.disabled = isComplete;
			button.classList.toggle("complete", isComplete);
			button.classList.toggle("has-note", !isComplete && notes.has(value));

			if (isComplete) {
				button.setAttribute("title", "All 9 placed");
			} else {
				button.removeAttribute("title");
			}
		}

		const clearButton = this.buttons.get("clear");
		if (clearButton) {
			clearButton.disabled = false;
			clearButton.classList.remove("complete", "has-note");
			clearButton.removeAttribute("title");
		}
	}

	private setVisibility(isVisible: boolean): void {
		if (!this.container) return;

		if (isVisible) {
			this.clearHideTimer();
			document.body.classList.add("numpad-visible");
			this.container.classList.remove("hidden");
			requestAnimationFrame(() => {
				this.container?.classList.add("show");
			});
		} else {
			this.container.classList.remove("show");
			document.body.classList.remove("numpad-visible");
			this.clearHideTimer();
			this.hideTimer = window.setTimeout(() => {
				this.container?.classList.add("hidden");
			}, 300);
		}
	}

	private clearHideTimer(): void {
		if (this.hideTimer !== null) {
			clearTimeout(this.hideTimer);
			this.hideTimer = null;
		}
	}

	private getNumericKey(event: KeyboardEvent): number | null {
		if (event.key >= "1" && event.key <= "9") {
			return parseInt(event.key, 10);
		}

		const digitMatch = event.code.match(/^Digit([1-9])$/);
		if (digitMatch) {
			return parseInt(digitMatch[1], 10);
		}

		const numpadMatch = event.code.match(/^Numpad([1-9])$/);
		if (numpadMatch) {
			return parseInt(numpadMatch[1], 10);
		}

		return null;
	}
}
