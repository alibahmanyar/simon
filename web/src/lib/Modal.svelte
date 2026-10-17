<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import type { HTMLDialogAttributes } from 'svelte/elements';

	// A native modal <dialog> that covers the viewport. Render it inside an {#if} and
	// close it by unmounting; the browser traps focus and makes the page behind inert.
	interface Props extends Omit<HTMLDialogAttributes, 'onclose'> {
		/** The user dismissed the dialog with Escape or a click on the backdrop */
		onclose?: () => void;
		/** When false, Escape and backdrop clicks are ignored, e.g. while a request runs */
		dismissible?: boolean;
		children: Snippet;
	}

	let { onclose, dismissible = true, children, class: className = '', ...rest }: Props = $props();

	let dialog: HTMLDialogElement;
	let pressedBackdrop = false;

	onMount(() => {
		const opener = document.activeElement;
		dialog.showModal();
		return () => {
			// Removing an open dialog skips the browser's own focus restore
			if (opener instanceof HTMLElement && opener.isConnected) opener.focus();
		};
	});

	function dismiss() {
		if (dismissible) onclose?.();
	}
</script>

<dialog
	bind:this={dialog}
	class="modal-root {className}"
	{...rest}
	oncancel={(e) => {
		e.preventDefault();
		dismiss();
	}}
	onclose={() => onclose?.()}
	onpointerdown={(e) => (pressedBackdrop = e.target === dialog)}
	onclick={(e) => {
		// Only a press and release both on the backdrop counts, not a text selection
		// that started inside the dialog
		if (pressedBackdrop && e.target === dialog) dismiss();
		pressedBackdrop = false;
	}}
>
	{@render children()}
</dialog>
