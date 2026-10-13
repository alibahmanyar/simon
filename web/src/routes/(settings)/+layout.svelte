<script>
	import '$lib/style.css';
	let { children } = $props();
	import { page } from '$app/state';
	import TopBar from '$lib/TopBar.svelte';
</script>

<TopBar>
	{#snippet tabs()}
		<a class="tab" class:active={page.url.pathname.endsWith('/notif_methods')} href="notif_methods"
			>Notification Methods</a
		>
		<a class="tab" class:active={page.url.pathname.endsWith('/alerts')} href="alerts">Alerts</a>
	{/snippet}
	{#snippet actions()}
		<a class="icon-btn" href="./" title="Back to dashboard">
			<svg
				xmlns="http://www.w3.org/2000/svg"
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M19 12H5"></path>
				<path d="M12 19l-7-7 7-7"></path>
			</svg>
			<span class="icon-btn-label">Dashboard</span>
		</a>
	{/snippet}
</TopBar>
<div class="dashboard">
	{@render children()}
</div>
