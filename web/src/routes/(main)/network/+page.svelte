<script lang="ts">
	import { formatBytes, formatBytesPerSecond, isVirtualInterface } from '$lib/utils.svelte';
	import { gdata } from '$lib/general_socket.svelte';
	import type { NetworkInterface } from '$lib/types';
	import Switch from '$lib/Switch.svelte';

	let showAllInterfaces = $state(false);

	// Rates come from the previous sample, so they're available on the first render
	// and toggling the switch doesn't reset them
	let sortedInterfaces: NetworkInterface[] = $derived.by(() => {
		const current = gdata.data?.current;
		if (!current?.net?.interfaces) return [];

		const previous = gdata.data?.prevDataPoints.at(-1);
		const elapsed =
			previous?.receivedAt && current.receivedAt
				? (current.receivedAt - previous.receivedAt) / 1000
				: 0;

		return current.net.interfaces
			.filter((iface) => showAllInterfaces || !isVirtualInterface(iface.name))
			.map((iface) => {
				const before =
					elapsed > 0 ? previous?.net.interfaces.find((p) => p.name === iface.name) : undefined;
				return {
					name: iface.name,
					rx: iface.rx,
					tx: iface.tx,
					receiveRate: before ? Math.max(0, (iface.rx - before.rx) / elapsed) : undefined,
					transmitRate: before ? Math.max(0, (iface.tx - before.tx) / elapsed) : undefined
				};
			})
			.sort((a, b) => a.name.localeCompare(b.name));
	});
</script>

{#if gdata.data?.current}
	<div class="card">
		<p class="card-title">Network Interfaces</p>
		<div class="chart-controls">
			<Switch label="Show All Interfaces" bind:checked={showAllInterfaces} />
		</div>
		<div id="network-interfaces">
			{#if sortedInterfaces.length > 0}
				{#each sortedInterfaces as iface (iface.name)}
					<div class="network-interface">
						<p class="card-title">{iface.name}</p>
						<div class="info-grid-0">
							<div class="info-item">
								<span class="info-label">Received:</span>
								<span class="info-value">{formatBytes(iface.rx)}</span>
							</div>
							<div class="info-item">
								<span class="info-label">Transmitted:</span>
								<span class="info-value">{formatBytes(iface.tx)}</span>
							</div>
							<div class="info-item">
								<span class="info-label">Receive Rate:</span>
								<span class="info-value"
									>{iface.receiveRate === undefined
										? '—'
										: formatBytesPerSecond(iface.receiveRate)}</span
								>
							</div>
							<div class="info-item">
								<span class="info-label">Transmit Rate:</span>
								<span class="info-value"
									>{iface.transmitRate === undefined
										? '—'
										: formatBytesPerSecond(iface.transmitRate)}</span
								>
							</div>
						</div>
					</div>
				{/each}
			{:else}
				<div class="info-item">
					<span>No network interfaces found</span>
				</div>
			{/if}
		</div>
	</div>

	<!-- Network rate chart -->
	<!-- <div class="chart-card">
		<p class="card-title">Network Transfer Rates</p>
		<div class="chart-controls">
			<select
				id="networkInterfaceSelect"
				bind:value={selectedInterface}
				onchange={handleInterfaceSelect}
			>
				<option value="">Select an interface</option>
				{#each sortedInterfaces as iface}
					<option value={iface.name}>{iface.name}</option>
				{/each}
			</select>
		</div>
		<div style="min-height: 30vh;"></div>
	</div> -->
{:else}
	<div class="loading">
		<div class="spinner"></div>
		<p>Waiting for system info...</p>
	</div>
{/if}
