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
		<a class="tab home-button" href="/notif_methods" style="margin-left: auto;">Settings</a>
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


