<script lang="ts">
	import type { Snippet } from 'svelte';
	import { asset } from '$app/paths';
	import ThemeToggle from '$lib/ThemeToggle.svelte';

	let { tabs, actions }: { tabs?: Snippet; actions?: Snippet } = $props();
</script>

<header class="topbar">
	<div class="topbar-inner">
		<a class="brand" href="./" aria-label="Simon dashboard">
			<img src={asset('/favicon.png')} alt="" width="28" height="28" />
			<span class="brand-name">Simon</span>
		</a>
		<nav class="tabs">
			{@render tabs?.()}
		</nav>
		<div class="topbar-actions">
			{@render actions?.()}
			<ThemeToggle />
		</div>
	</div>
</header>
