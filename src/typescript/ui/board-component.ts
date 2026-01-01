import { UIComponent, bindEvent } from "./component.js";
import { createElement } from "../utils.js";

export interface BoardCellView {
	index: number;
	value: number | null;
	notes: number[];
	isGiven: boolean;
	isUserInput: boolean;
	isInvalid: boolean;
	isFocused: boolean;
	isSelected: boolean;
	isMatching: boolean;
	isHinted: boolean;
}

export interface BoardViewState {
	cells: BoardCellView[];
}

export interface BoardComponentContext {
	onCellSelect: (cellIndex: number) => void;
}

export class BoardComponent
	implements UIComponent<BoardComponentContext, BoardViewState>
{
	private container: HTMLElement | null = null;
	private cells: HTMLElement[] = [];
	private cleanup: Array<() => void> = [];
	private context: BoardComponentContext | null = null;

	init(context: BoardComponentContext): void {
		this.context = context;
	}

	mount(container: HTMLElement): void {
		this.unmount();
		this.container = container;
		this.container.innerHTML = "";
		this.cells = [];

		for (let i = 0; i < 81; i++) {
			const cell = createElement("div", {
				className: "cell",
				tabIndex: 0,
				dataset: { index: i.toString() },
			});

			const activateCell = (event?: Event) => {
				if (event?.type === "touchend") {
					event.preventDefault();
				}
				this.context?.onCellSelect(i);
			};

			this.cleanup.push(
				bindEvent(cell, "click", activateCell),
				bindEvent(cell, "touchend", activateCell),
				bindEvent(cell, "keydown", (event: KeyboardEvent) => {
					if (event.key === "Enter" || event.key === " ") {
						event.preventDefault();
						this.context?.onCellSelect(i);
					}
				})
			);

			this.cells.push(cell);
			this.container.appendChild(cell);
		}
	}

	update(state: BoardViewState): void {
		state.cells.forEach((cellState) => {
			const cell = this.cells[cellState.index];
			if (!cell) return;

			cell.classList.toggle("given", cellState.isGiven);
			cell.classList.toggle("user-input", cellState.isUserInput);
			cell.classList.toggle("invalid", cellState.isInvalid);
			cell.classList.toggle("focused", cellState.isFocused);
			cell.classList.toggle("selected", cellState.isSelected);
			cell.classList.toggle("matching-value", cellState.isMatching);
			cell.classList.toggle("hint-cell", cellState.isHinted);

			cell.innerHTML = "";

			if (cellState.value !== null) {
				cell.textContent = cellState.value.toString();
				return;
			}

			if (cellState.notes.length > 0) {
				const notesContainer = document.createElement("div");
				notesContainer.className = "cell-notes-display";
				notesContainer.style.position = "absolute";
				notesContainer.style.top = "2px";
				notesContainer.style.left = "2px";
				notesContainer.style.right = "2px";
				notesContainer.style.fontSize = "0.6em";
				notesContainer.style.display = "flex";
				notesContainer.style.flexWrap = "wrap";
				notesContainer.style.gap = "1px";
				notesContainer.style.pointerEvents = "none";

				cellState.notes.forEach((note) => {
					const noteSpan = document.createElement("span");
					noteSpan.textContent = note.toString();
					noteSpan.style.opacity = "0.7";
					notesContainer.appendChild(noteSpan);
				});

				cell.appendChild(notesContainer);
			}
		});
	}

	unmount(): void {
		this.cleanup.forEach((cleanup) => cleanup());
		this.cleanup = [];
		this.cells = [];
		if (this.container) {
			this.container.innerHTML = "";
		}
		this.container = null;
	}
}
