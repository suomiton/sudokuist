import { UIComponent, bindEvent } from "./component.js";
import { createElement } from "../utils.js";

export interface SharePuzzleViewState {
	link: string;
	isVisible: boolean;
}

export interface SharePuzzleComponentContext {
	onCopy?: (link: string) => void;
	onCopyError?: (link: string, error: unknown) => void;
}

export class SharePuzzleComponent
	implements UIComponent<SharePuzzleComponentContext, SharePuzzleViewState>
{
	private container: HTMLElement | null = null;
	private input: HTMLInputElement | null = null;
	private copyButton: HTMLButtonElement | null = null;
	private cleanup: Array<() => void> = [];
	private context: SharePuzzleComponentContext | null = null;

	/** Save callbacks for copy feedback. */
	init(context: SharePuzzleComponentContext): void {
		this.context = context;
	}

	/** Build the share link UI and wire copy behavior. */
	mount(container: HTMLElement): void {
		this.unmount();
		this.container = container;
		this.container.innerHTML = "";

		const header = createElement("div", { className: "share-header" });
		header.appendChild(
			createElement("span", { className: "share-label" }, "Share this puzzle:")
		);

		const linkContainer = createElement("div", { className: "share-link-container" });
		this.input = createElement("input", {
			type: "text",
			className: "share-link",
			id: "share-link",
			readOnly: true,
		});
		this.copyButton = createElement("button", {
			className: "copy-btn",
			id: "copy-link-btn",
			textContent: "Copy",
		});

		linkContainer.appendChild(this.input);
		linkContainer.appendChild(this.copyButton);

		this.container.appendChild(header);
		this.container.appendChild(linkContainer);

		if (this.copyButton) {
			this.cleanup.push(
				bindEvent(this.copyButton, "click", () => this.handleCopy())
			);
		}
	}

	/** Update the share link value and visibility state. */
	update(state: SharePuzzleViewState): void {
		if (this.container) {
			this.container.classList.toggle("hidden", !state.isVisible);
		}
		if (this.input) {
			this.input.value = state.link;
		}
	}

	/** Remove event listeners and reset the share UI. */
	unmount(): void {
		this.cleanup.forEach((cleanup) => cleanup());
		this.cleanup = [];
		if (this.container) {
			this.container.innerHTML = "";
		}
		this.container = null;
		this.input = null;
		this.copyButton = null;
	}

	/** Copy the current share link to the clipboard with UI feedback. */
	private async handleCopy(): Promise<void> {
		if (!this.input || !this.copyButton) return;

		const link = this.input.value;
		try {
			await navigator.clipboard.writeText(link);
			this.copyButton.textContent = "Copied!";
			this.copyButton.classList.add("copied");
			this.context?.onCopy?.(link);

			setTimeout(() => {
				if (this.copyButton) {
					this.copyButton.textContent = "Copy";
					this.copyButton.classList.remove("copied");
				}
			}, 2000);
		} catch (error) {
			this.context?.onCopyError?.(link, error);
			this.input.select();
			this.input.setSelectionRange(0, 99999);
			alert("Link selected. Press Ctrl+C to copy.");
		}
	}
}
