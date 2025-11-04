<script>
	import '$lib/style.css';
	let { children } = $props();
	import { page } from '$app/state';
	import {
		gdata,
		open_ws as general_ws_open,
		close_ws as general_ws_close
	} from '$lib/general_socket.svelte';
	import {
		open_ws as docker_ws_open,
		close_ws as docker_ws_close
	} from '$lib/docker_socket.svelte';
	import { wsStatus } from '$lib/types';
	import { onMount, onDestroy } from 'svelte';
	import { capabilities, updateCapabilities } from '$lib/utils.svelte';

	onMount(() => {
		updateCapabilities();
		setTimeout(general_ws_open, 2);
		setTimeout(docker_ws_open, 2);

		return () => {
			general_ws_close();
			docker_ws_close();
		};
	});

	onDestroy(() => {
		general_ws_close();
		docker_ws_close();
	});
</script>

<h1>Simon</h1>
<div class="dashboard">
	{#if gdata.status === wsStatus.CONNECTED}
		<nav class="tabs">
			<a class="tab" class:active={page.url.pathname === '/'} href="/">Overview</a>
			{#if capabilities?.disk}
				<a class="tab" class:active={page.url.pathname === '/storage'} href="storage">Storage</a>
			{/if}
			{#if capabilities?.network}
				<a class="tab" class:active={page.url.pathname === '/network'} href="network">Network</a>
			{/if}
			<!-- <a class="tab" class:active={page.url.pathname==='/processes'} href="/processes">Processes</a> -->
			{#if capabilities?.docker}
				<a class="tab" class:active={page.url.pathname === '/docker'} href="docker">Docker</a>
			{/if}
			<a class="tab" class:active={page.url.pathname === '/graphs'} href="graphs"
				>Historical Charts</a
			>
			{#if capabilities?.file_serving}
				<a class="tab" class:active={page.url.pathname === '/files'} href="/files">Files</a>
			{/if}
			<a
				class="tab home-button"
				href="/notif_methods"
				style="margin-left: auto;"
				aria-label="Settings"
			>
				<span style="display: flex; align-items: center; gap:5px;">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						width="16"
						height="16"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"></path>
						<path d="M13.73 21a2 2 0 0 1-3.46 0"></path>
					</svg>
					Alerts
				</span>
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
