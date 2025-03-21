<script>
	import '$lib/style.css';
	let { children } = $props();
    import { page } from '$app/state';
    import { gdata } from '$lib/general_socket.svelte';
	import { wsStatus } from '$lib/types';
</script>
<h1>Simon</h1>
<div class="dashboard">
	{#if gdata.status === wsStatus.CONNECTED}
	<nav class="tabs">
		<a class="tab" class:active={page.url.pathname==='/'} href="/">Overview</a>
		<a class="tab" class:active={page.url.pathname==='/storage'} href="/storage">Storage</a>
		<a class="tab" class:active={page.url.pathname==='/network'} href="/network">Network</a>
        <!-- <a class="tab" class:active={page.url.pathname==='/processes'} href="/processes">Processes</a> -->
		<a class="tab" class:active={page.url.pathname==='/docker'} href="/docker">Docker</a>
		<a class="tab" class:active={page.url.pathname==='/graphs'} href="/graphs">Historical Charts</a>
		<a class="tab home-button" href="/notif_methods" style="margin-left: auto;" aria-label="Settings">
			<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<circle cx="12" cy="12" r="3"></circle>
				<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
			</svg>
		</a>
	</nav>
    {@render children()}
	{:else if gdata.status === wsStatus.INIT}
		<div class="loading">
			<div class="spinner"></div>
			<p>Initializing web socket...</p>
		</div>
	{:else if gdata.status === wsStatus.WAITING}
		<div class="loading">
			<div class="spinner"></div>
			<p>Waiting for system info...</p>
		</div>
	{:else if gdata.status === wsStatus.DISCONNECTED || gdata.status === wsStatus.ERROR}
		<div class="error-container">
			<p>Connection failed</p>
			<p>Could not connect to the data service. Please check your network connection.</p>
		</div>
	{/if}
</div>


