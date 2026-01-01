import { bindEvent } from "./component.js";
import {
	BoardComponent,
	BoardComponentContext,
	BoardViewState,
} from "./board-component.js";
import { DifficultySelectionComponent } from "./difficulty-selection-component.js";
import {
	HintComponent,
	HintComponentContext,
	HintViewState,
} from "./hint-component.js";
import {
	MainMenuComponent,
	MainMenuComponentContext,
	MainMenuViewState,
} from "./main-menu-component.js";
import {
	NumpadComponent,
	NumpadComponentContext,
	NumpadViewState,
} from "./numpad-component.js";
import {
	ScoreboardComponent,
	ScoreboardComponentContext,
	ScoreboardViewState,
} from "./scoreboard-component.js";
import {
	SharePuzzleComponent,
	SharePuzzleComponentContext,
	SharePuzzleViewState,
} from "./share-puzzle-component.js";

export interface UIShellContainers {
	startScreen: HTMLElement;
	sudokuContainer: HTMLElement;
	scoreboardContainer: HTMLElement;
	board: HTMLElement;
	numpad: HTMLElement;
	shareContainer: HTMLElement;
	hintContainer: HTMLElement;
	timerDisplay: HTMLElement;
	backButton?: HTMLElement | null;
}

export interface UIShellInit {
	menu: MainMenuComponentContext;
	scoreboard: ScoreboardComponentContext;
	board: BoardComponentContext;
	numpad: NumpadComponentContext;
	hint: HintComponentContext;
	share?: SharePuzzleComponentContext;
	onBackToMenu?: () => void;
}

export interface UIShellState {
	menu?: MainMenuViewState;
	scoreboard?: ScoreboardViewState;
	board?: BoardViewState;
	numpad?: NumpadViewState;
	share?: SharePuzzleViewState;
	hint?: HintViewState;
}

export class UIShell {
	private containers: UIShellContainers;
	private menuComponent = new MainMenuComponent();
	private scoreboardComponent = new ScoreboardComponent();
	private boardComponent = new BoardComponent();
	private numpadComponent = new NumpadComponent();
	private shareComponent = new SharePuzzleComponent();
	private hintComponent = new HintComponent();
	private difficultyComponent = new DifficultySelectionComponent();
	private backButtonCleanup: (() => void) | null = null;

	constructor(containers: UIShellContainers) {
		this.containers = containers;
	}

	initialize(handlers: UIShellInit): void {
		this.menuComponent.init(handlers.menu);
		this.scoreboardComponent.init(handlers.scoreboard);
		this.boardComponent.init(handlers.board);
		this.numpadComponent.init(handlers.numpad);
		this.shareComponent.init(handlers.share ?? {});
		this.hintComponent.init(handlers.hint);
		this.difficultyComponent.init({});

		this.menuComponent.mount(this.containers.startScreen);
		this.scoreboardComponent.mount(this.containers.scoreboardContainer);
		this.boardComponent.mount(this.containers.board);
		this.numpadComponent.mount(this.containers.numpad);
		this.shareComponent.mount(this.containers.shareContainer);
		this.hintComponent.mount(this.containers.hintContainer);
		this.difficultyComponent.mount(document.body);

		if (this.backButtonCleanup) {
			this.backButtonCleanup();
			this.backButtonCleanup = null;
		}

		if (handlers.onBackToMenu && this.containers.backButton) {
			this.backButtonCleanup = bindEvent(
				this.containers.backButton,
				"click",
				() => handlers.onBackToMenu?.()
			);
		}
	}

	requestDifficultySelection(): Promise<number | null> {
		return this.difficultyComponent.requestSelection();
	}

	showMenu(): void {
		this.containers.startScreen.classList.remove("hidden");
		this.containers.sudokuContainer.classList.add("hidden");
		this.containers.scoreboardContainer.classList.add("hidden");
	}

	showBoard(): void {
		this.containers.startScreen.classList.add("hidden");
		this.containers.sudokuContainer.classList.remove("hidden");
		this.containers.scoreboardContainer.classList.add("hidden");
	}

	showScoreboard(): void {
		this.containers.startScreen.classList.add("hidden");
		this.containers.sudokuContainer.classList.add("hidden");
		this.containers.scoreboardContainer.classList.remove("hidden");
	}

	render(state: UIShellState): void {
		if (state.menu) {
			this.menuComponent.update(state.menu);
		}
		if (state.scoreboard) {
			this.scoreboardComponent.update(state.scoreboard);
		}
		if (state.board) {
			this.boardComponent.update(state.board);
		}
		if (state.numpad) {
			this.numpadComponent.update(state.numpad);
		}
		if (state.share) {
			this.shareComponent.update(state.share);
		}
		if (state.hint) {
			this.hintComponent.update(state.hint);
		}
	}

	updateTimerDisplay(label: string): void {
		this.containers.timerDisplay.textContent = label;
	}
}
