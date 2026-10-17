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
	import TopBar from '$lib/TopBar.svelte';

	let errorMsg = $state('');
	let capsLoaded = $state(false);
	let now = $state(Date.now());

	async function loadCapabilities() {
		const data = await updateCapabilities();
		if (!data.success) {
			console.error('Failed to fetch capabilities:', data);
			errorMsg = data.error || 'Unknown error';
			return;
		}
		errorMsg = '';
		capsLoaded = true;
		// Only open Docker WebSocket if Docker capability is available
		if (capabilities.docker) {
			setTimeout(docker_ws_open, 2);
		}
	}

	onMount(() => {
		loadCapabilities();
		setTimeout(general_ws_open, 2);
	});

	onDestroy(() => {
		general_ws_close();
		docker_ws_close();
	});

	// The server may be down on first load; keep trying until it answers
	$effect(() => {
		if (!errorMsg) return;
		const timer = setInterval(loadCapabilities, 5000);
		return () => clearInterval(timer);
	});

	// Pages keep showing their last data while the connection is down; the banner says so
	let offline = $derived(gdata.status === wsStatus.DISCONNECTED || gdata.status === wsStatus.ERROR);

	$effect(() => {
		if (!offline) return;
		now = Date.now();
		const timer = setInterval(() => (now = Date.now()), 1000);
		return () => clearInterval(timer);
	});

	/** @param {number} ms */
	function formatAge(ms) {
		const seconds = Math.max(0, Math.round(ms / 1000));
		if (seconds < 60) return `${seconds} s`;
		if (seconds < 3600) return `${Math.floor(seconds / 60)} min`;
		return `${Math.floor(seconds / 3600)} h`;
	}
</script>

<TopBar>
	{#snippet tabs()}
		{#if capsLoaded}
			<a class="tab" class:active={page.url.pathname.endsWith('/')} href="./">Overview</a>
			{#if capabilities?.disk}
				<a class="tab" class:active={page.url.pathname.endsWith('/storage')} href="storage"
					>Storage</a
				>
			{/if}
			{#if capabilities?.network}
				<a class="tab" class:active={page.url.pathname.endsWith('/network')} href="network"
					>Network</a
				>
			{/if}
			<!-- <a class="tab" class:active={page.url.pathname==='/processes'} href="/processes">Processes</a> -->
			<!-- Docker and Files stay listed when unavailable; their pages explain how to enable them -->
			<a class="tab" class:active={page.url.pathname.endsWith('/docker')} href="docker">Docker</a>
			<a class="tab" class:active={page.url.pathname.endsWith('/graphs')} href="graphs"
				>Historical Charts</a
			>
			<a class="tab" class:active={page.url.pathname.endsWith('/files')} href="files">Files</a>
		{/if}
	{/snippet}
	{#snippet actions()}
		<a class="icon-btn" href="notif_methods" title="Alerts and notification settings">
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
				<path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"></path>
				<path d="M13.73 21a2 2 0 0 1-3.46 0"></path>
			</svg>
			<span class="icon-btn-label">Alerts</span>
		</a>
	{/snippet}
	{#snippet banner()}
		<!-- Live region exists up front so screen readers announce what appears in it -->
		<div role="status">
			{#if capsLoaded && offline}
				<div class="conn-banner">
					{#if gdata.lastMessageAt}
						Connection lost, reconnecting…
						<span aria-hidden="true">
							Showing data from {formatAge(now - gdata.lastMessageAt)} ago.
						</span>
					{:else}
						Can't reach the server, retrying…
					{/if}
				</div>
			{/if}
		</div>
	{/snippet}
</TopBar>
<div class="dashboard">
	{#if capsLoaded}
		{@render children()}
	{:else if errorMsg}
		<div class="error-container" role="alert">
			<p>Can't reach Simon</p>
			<p>{errorMsg}. Retrying automatically…</p>
		</div>
	{:else}
		<div class="loading">
			<div class="spinner"></div>
			<p>Connecting…</p>
		</div>
	{/if}
</div>
