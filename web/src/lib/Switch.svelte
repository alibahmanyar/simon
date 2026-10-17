<script lang="ts">
	// An on/off switch with a clickable text label, backed by a native checkbox.
	// `spread` pushes the switch to the far end of the row instead of next to the label.
	let {
		checked = $bindable(false),
		label,
		spread = false,
		onchange
	}: {
		checked?: boolean;
		label: string;
		spread?: boolean;
		onchange?: (e: Event) => void;
	} = $props();
</script>

<label class="switch-row" class:spread>
	<span class="switch-text">{label}</span>
	<span class="switch">
		<input type="checkbox" role="switch" bind:checked {onchange} />
		<span class="slider" aria-hidden="true"></span>
	</span>
</label>
