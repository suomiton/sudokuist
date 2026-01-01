/**
 * Defines the lifecycle contract for UI components.
 */
export interface UIComponent<Context = unknown, State = unknown> {
	/** Initialize with dependencies and state needed to render. */
	init(context: Context): void;
	/** Attach the component's DOM to the provided container. */
	mount(container: HTMLElement): void;
	/** Re-render the component for the provided state. */
	update(state: State): void;
	/** Tear down DOM and event listeners. */
	unmount(): void;
}

/**
 * Wire an event handler and return an explicit cleanup function.
 */
export function bindEvent<E extends Event>(
	target: EventTarget,
	type: string,
	handler: (event: E) => void,
	options?: boolean | AddEventListenerOptions
): () => void {
	target.addEventListener(type, handler as EventListener, options);
	return () => target.removeEventListener(type, handler as EventListener, options);
}
