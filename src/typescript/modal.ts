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
	private progressBar: HTMLElement | null = null;
	private progressMessage: HTMLElement | null = null;
	private progressDetails: HTMLElement | null = null;

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
	 * Show a loading modal with an indeterminate progress bar
	 */
	showLoading(title: string, message: string): void {
		this.clearModalImmediate();

		this.overlay = document.createElement("div");
		this.overlay.className = "modal-overlay modal-overlay-visible";

		this.modal = document.createElement("div");
		this.modal.className = "modal-container modal-info modal-loading";

		const content = document.createElement("div");
		content.className = "modal-content";

		const header = document.createElement("div");
		header.className = "modal-header";

		const modalTitle = document.createElement("h3");
		modalTitle.className = "modal-title";
		modalTitle.textContent = title;
		header.appendChild(modalTitle);

		const body = document.createElement("div");
		body.className = "modal-body";

		const text = document.createElement("p");
		text.className = "modal-message";
		text.textContent = message;
		body.appendChild(text);
		this.progressMessage = text;

		const progress = document.createElement("div");
		progress.className = "modal-progress";

		const progressBar = document.createElement("div");
		progressBar.className = "modal-progress-bar";
		progressBar.style.width = "0%";
		this.progressBar = progressBar;

		progress.appendChild(progressBar);
		body.appendChild(progress);

		const details = document.createElement("div");
		details.className = "modal-progress-details";
		details.textContent = "";
		this.progressDetails = details;
		body.appendChild(details);

		content.appendChild(header);
		content.appendChild(body);
		this.modal.appendChild(content);
		this.overlay.appendChild(this.modal);

		document.body.appendChild(this.overlay);
		requestAnimationFrame(() => {
			this.modal?.classList.add("modal-container-visible");
		});
	}

	hideLoading(): void {
		this.closeModal();
	}

	setLoadingProgress(percent: number, stage?: string): void {
		if (this.progressBar) {
			const clamped = Math.max(0, Math.min(100, percent));
			this.progressBar.style.width = `${clamped}%`;
		}

		if (stage && this.progressMessage) {
			this.progressMessage.textContent = stage;
		}
	}

	setLoadingDetails(details: string): void {
		if (this.progressDetails) {
			this.progressDetails.textContent = details;
		}
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
	 * Close and remove the modal
	 */
	private closeModal(): void {
		if (!this.overlay) return;

		const overlay = this.overlay;
		const modal = this.modal;

		// Animate out
		overlay.classList.remove("modal-overlay-visible");
		modal?.classList.remove("modal-container-visible");

		// Remove from DOM after animation
		setTimeout(() => {
			if (overlay && overlay.parentNode) {
				overlay.parentNode.removeChild(overlay);
			}
		}, 300);

		this.overlay = null;
		this.modal = null;
		this.progressBar = null;
		this.progressMessage = null;
		this.progressDetails = null;
	}

	private clearModalImmediate(): void {
		if (this.overlay && this.overlay.parentNode) {
			this.overlay.parentNode.removeChild(this.overlay);
		}
		this.overlay = null;
		this.modal = null;
		this.progressBar = null;
		this.progressMessage = null;
		this.progressDetails = null;
	}
}

// Create a global modal instance
export const modal = new Modal();
