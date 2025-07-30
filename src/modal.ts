/**
 * Modal Component - Custom modal dialogs with overlay
 */

export interface ModalConfig {
	title: string;
	message: string;
	type?: "info" | "warning" | "success" | "confirmation";
	showCancel?: boolean;
	confirmText?: string;
	cancelText?: string;
	onConfirm?: () => void;
	onCancel?: () => void;
}

export class Modal {
	private overlay: HTMLElement | null = null;
	private modal: HTMLElement | null = null;

	/**
	 * Show a modal dialog
	 */
	show(config: ModalConfig): Promise<boolean> {
		return new Promise((resolve) => {
			this.createModal(config, resolve);
		});
	}

	/**
	 * Show a confirmation dialog
	 */
	confirm(title: string, message: string): Promise<boolean> {
		return this.show({
			title,
			message,
			type: "confirmation",
			showCancel: true,
			confirmText: "Yes",
			cancelText: "No",
		});
	}

	/**
	 * Show a success dialog (for puzzle completion)
	 */
	success(
		title: string,
		message: string,
		onConfirm?: () => void
	): Promise<boolean> {
		return this.show({
			title,
			message,
			type: "success",
			showCancel: false,
			confirmText: "OK",
			onConfirm,
		});
	}

	/**
	 * Show an info dialog
	 */
	info(title: string, message: string): Promise<boolean> {
		return this.show({
			title,
			message,
			type: "info",
			showCancel: false,
			confirmText: "OK",
		});
	}

	/**
	 * Show a difficulty selection dialog
	 */
	selectDifficulty(): Promise<number | null> {
		return new Promise((resolve) => {
			this.createDifficultyModal(resolve);
		});
	}

	/**
	 * Create and display the modal
	 */
	private createModal(
		config: ModalConfig,
		resolve: (value: boolean) => void
	): void {
		// Create overlay
		this.overlay = document.createElement("div");
		this.overlay.className = "modal-overlay";

		// Create modal container
		this.modal = document.createElement("div");
		this.modal.className = `modal-container modal-${config.type || "info"}`;

		// Create modal content
		const content = document.createElement("div");
		content.className = "modal-content";

		// Modal header
		const header = document.createElement("div");
		header.className = "modal-header";

		const title = document.createElement("h3");
		title.className = "modal-title";
		title.textContent = config.title;

		header.appendChild(title);

		// Modal body
		const body = document.createElement("div");
		body.className = "modal-body";

		const message = document.createElement("p");
		message.className = "modal-message";
		message.innerHTML = config.message; // Allow HTML for formatting

		body.appendChild(message);

		// Modal footer
		const footer = document.createElement("div");
		footer.className = "modal-footer";

		// Create buttons
		const confirmBtn = document.createElement("button");
		confirmBtn.className = "modal-btn modal-btn-confirm";
		confirmBtn.textContent = config.confirmText || "OK";
		confirmBtn.addEventListener("click", () => {
			config.onConfirm?.();
			this.closeModal();
			resolve(true);
		});

		footer.appendChild(confirmBtn);

		if (config.showCancel) {
			const cancelBtn = document.createElement("button");
			cancelBtn.className = "modal-btn modal-btn-cancel";
			cancelBtn.textContent = config.cancelText || "Cancel";
			cancelBtn.addEventListener("click", () => {
				config.onCancel?.();
				this.closeModal();
				resolve(false);
			});
			footer.appendChild(cancelBtn);
		}

		// Assemble modal
		content.appendChild(header);
		content.appendChild(body);
		content.appendChild(footer);
		this.modal.appendChild(content);
		this.overlay.appendChild(this.modal);

		// Add to DOM
		document.body.appendChild(this.overlay);

		// Handle ESC key and overlay click
		this.setupEventListeners(config, resolve);

		// Focus the confirm button
		setTimeout(() => confirmBtn.focus(), 100);

		// Animate in
		requestAnimationFrame(() => {
			this.overlay?.classList.add("modal-overlay-visible");
			this.modal?.classList.add("modal-container-visible");
		});
	}

	/**
	 * Create and display the difficulty selection modal
	 */
	private createDifficultyModal(resolve: (value: number | null) => void): void {
		// Create overlay
		this.overlay = document.createElement("div");
		this.overlay.className = "modal-overlay";

		// Create modal container
		this.modal = document.createElement("div");
		this.modal.className = "modal-container modal-info";

		// Create modal content
		const content = document.createElement("div");
		content.className = "modal-content";

		// Modal header
		const header = document.createElement("div");
		header.className = "modal-header";

		const title = document.createElement("h3");
		title.className = "modal-title";
		title.textContent = "Select Difficulty";

		header.appendChild(title);

		// Modal body
		const body = document.createElement("div");
		body.className = "modal-body";

		const description = document.createElement("p");
		description.className = "modal-message";
		description.textContent = "Choose your preferred difficulty level:";
		body.appendChild(description);

		// Difficulty selection container
		const difficultyContainer = document.createElement("div");
		difficultyContainer.className = "difficulty-selection";

		let selectedDifficulty = 1; // Default to easiest

		// Create difficulty options
		const difficulties = [
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
				name: "Very Hard",
				stars: 5,
				description: "17-24 clues, expert level",
			},
		];

		difficulties.forEach((diff) => {
			const option = document.createElement("div");
			option.className = "difficulty-option";
			if (diff.level === 1) option.classList.add("selected");

			// Stars display
			const starsContainer = document.createElement("div");
			starsContainer.className = "difficulty-stars";

			for (let i = 1; i <= 5; i++) {
				const star = document.createElement("span");
				star.className = i <= diff.stars ? "star filled" : "star empty";
				star.textContent = "★";
				starsContainer.appendChild(star);
			}

			// Difficulty info
			const info = document.createElement("div");
			info.className = "difficulty-info";

			const name = document.createElement("div");
			name.className = "difficulty-name";
			name.textContent = diff.name;

			const desc = document.createElement("div");
			desc.className = "difficulty-description";
			desc.textContent = diff.description;

			info.appendChild(name);
			info.appendChild(desc);

			option.appendChild(starsContainer);
			option.appendChild(info);

			// Click handler
			option.addEventListener("click", () => {
				// Remove selected from all options
				difficultyContainer
					.querySelectorAll(".difficulty-option")
					.forEach((opt) => {
						opt.classList.remove("selected");
					});
				// Add selected to clicked option
				option.classList.add("selected");
				selectedDifficulty = diff.level;
			});

			difficultyContainer.appendChild(option);
		});

		body.appendChild(difficultyContainer);

		// Modal footer
		const footer = document.createElement("div");
		footer.className = "modal-footer";

		// Start button
		const startBtn = document.createElement("button");
		startBtn.className = "modal-btn modal-btn-confirm";
		startBtn.textContent = "Start Game";
		startBtn.addEventListener("click", () => {
			this.closeModal();
			resolve(selectedDifficulty);
		});

		// Cancel button
		const cancelBtn = document.createElement("button");
		cancelBtn.className = "modal-btn modal-btn-cancel";
		cancelBtn.textContent = "Cancel";
		cancelBtn.addEventListener("click", () => {
			this.closeModal();
			resolve(null);
		});

		footer.appendChild(startBtn);
		footer.appendChild(cancelBtn);

		// Assemble modal
		content.appendChild(header);
		content.appendChild(body);
		content.appendChild(footer);
		this.modal.appendChild(content);
		this.overlay.appendChild(this.modal);

		// Add to DOM
		document.body.appendChild(this.overlay);

		// Handle ESC key and overlay click for difficulty modal
		this.setupDifficultyEventListeners(resolve);

		// Focus the start button
		setTimeout(() => startBtn.focus(), 100);

		// Animate in
		requestAnimationFrame(() => {
			this.overlay?.classList.add("modal-overlay-visible");
			this.modal?.classList.add("modal-container-visible");
		});
	}

	/**
	 * Setup event listeners for modal interactions
	 */
	private setupEventListeners(
		config: ModalConfig,
		resolve: (value: boolean) => void
	): void {
		if (!this.overlay) return;

		// ESC key handler
		const handleKeydown = (e: KeyboardEvent) => {
			if (e.key === "Escape") {
				config.onCancel?.();
				this.closeModal();
				resolve(false);
				document.removeEventListener("keydown", handleKeydown);
			}
		};

		// Overlay click handler
		const handleOverlayClick = (e: MouseEvent) => {
			if (e.target === this.overlay) {
				config.onCancel?.();
				this.closeModal();
				resolve(false);
			}
		};

		document.addEventListener("keydown", handleKeydown);
		this.overlay.addEventListener("click", handleOverlayClick);
	}

	/**
	 * Setup event listeners for difficulty modal
	 */
	private setupDifficultyEventListeners(
		resolve: (value: number | null) => void
	): void {
		if (!this.overlay) return;

		// ESC key handler
		const handleKeydown = (e: KeyboardEvent) => {
			if (e.key === "Escape") {
				this.closeModal();
				resolve(null);
				document.removeEventListener("keydown", handleKeydown);
			}
		};

		// Overlay click handler
		const handleOverlayClick = (e: MouseEvent) => {
			if (e.target === this.overlay) {
				this.closeModal();
				resolve(null);
			}
		};

		document.addEventListener("keydown", handleKeydown);
		this.overlay.addEventListener("click", handleOverlayClick);
	}

	/**
	 * Close and remove the modal
	 */
	private closeModal(): void {
		if (!this.overlay) return;

		// Animate out
		this.overlay.classList.remove("modal-overlay-visible");
		this.modal?.classList.remove("modal-container-visible");

		// Remove from DOM after animation
		setTimeout(() => {
			if (this.overlay && this.overlay.parentNode) {
				this.overlay.parentNode.removeChild(this.overlay);
			}
			this.overlay = null;
			this.modal = null;
		}, 300);
	}
}

// Create a global modal instance
export const modal = new Modal();
