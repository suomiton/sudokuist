/**
 * Utility functions for the Sudoku application
 */

/**
 * Utility function to create DOM elements with props and children
 * Provides type safety for HTML element creation
 */
export function createElement<K extends keyof HTMLElementTagNameMap>(
	tag: K,
	props?: Partial<HTMLElementTagNameMap[K]>,
	...children: (Node | string)[]
): HTMLElementTagNameMap[K] {
	const element = document.createElement(tag);

	if (props) {
		// Handle dataset separately since it's read-only
		const { dataset, ...otherProps } = props as any;

		// Assign other properties
		Object.assign(element, otherProps);

		// Handle dataset manually
		if (dataset) {
			for (const [key, value] of Object.entries(dataset)) {
				element.dataset[key] = value as string;
			}
		}
	}

	children.forEach((child) => {
		if (typeof child === "string") {
			element.appendChild(document.createTextNode(child));
		} else {
			element.appendChild(child);
		}
	});

	return element;
}

/**
 * Generate a UUID v4 for game identification
 */
export function generateUUID(): string {
	return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, function (c) {
		const r = (Math.random() * 16) | 0;
		const v = c === "x" ? r : (r & 0x3) | 0x8;
		return v.toString(16);
	});
}

/**
 * Parse URL parameters for shareable puzzle functionality
 * Returns seed and difficulty if both are valid, null otherwise
 */
export function getShareParams(): { seed: number; difficulty: number } | null {
	const p = new URLSearchParams(location.search);
	const s = Number(p.get("seed"));
	const d = Number(p.get("difficulty"));

	// Validate that seed is an integer and difficulty is between 1-9
	return Number.isInteger(s) && d >= 1 && d <= 9
		? { seed: s, difficulty: d }
		: null;
}
