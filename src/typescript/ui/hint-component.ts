import { UIComponent, bindEvent } from "./component.js";
import { createElement } from "../utils.js";
import { modal } from "../modal.js";

export interface HintViewState {
	isEnabled: boolean;
	label: string;
}

export interface HintComponentContext {
	onConfirmHint: () => void;
}

export class HintComponent
	implements UIComponent<HintComponentContext, HintViewState>
{
	private container: HTMLElement | null = null;
	private button: HTMLButtonElement | null = null;
	private cleanup: Array<() => void> = [];
	private context: HintComponentContext | null = null;

	/** Store the callback to execute when a hint is confirmed. */
	init(context: HintComponentContext): void {
		this.context = context;
	}

	/** Build the hint button and attach the confirmation flow. */
	mount(container: HTMLElement): void {
		this.unmount();
		this.container = container;
		this.container.innerHTML = "";

		this.button = createElement("button", {
			className: "control-btn",
			id: "btn-solve",
			textContent: "Hint",
		});

		this.container.appendChild(this.button);

		this.cleanup.push(
			bindEvent(this.button, "click", () => this.handleHintClick())
		);
	}

	/** Update the hint button label and enabled state. */
	update(state: HintViewState): void {
		if (this.button) {
			this.button.disabled = !state.isEnabled;
			this.button.textContent = state.label;
		}
	}

	/** Remove listeners and clear the hint container. */
	unmount(): void {
		this.cleanup.forEach((cleanup) => cleanup());
		this.cleanup = [];
		if (this.container) {
			this.container.innerHTML = "";
		}
		this.container = null;
		this.button = null;
	}

	/** Confirm hint usage and notify the caller on approval. */
	private async handleHintClick(): Promise<void> {
		const confirmed = await modal.confirm(
			"Use Hint",
			"Do you want to reveal one number? This will count as a hint used."
		);

		if (confirmed) {
			this.context?.onConfirmHint();
		}
	}
}
