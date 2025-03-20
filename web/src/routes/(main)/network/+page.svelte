<script lang="ts">
	import { formatBytes, formatBytesPerSecond } from '$lib/utils';
	import { gdata } from '$lib/general_socket.svelte';
	import Chart from '$lib/Chart.svelte';

	// Network data tracking
	let networkInterfaces: { [key: string]: any } = {};
	let previousNetworkStats: { [key: string]: any } = {};
	let selectedInterface: string = $state('');
	let sortedInterfaces: any[] = $state([]);

	let showAllInterfaces = $state(false);

	// Process network data when it updates
	$effect(() => {
		if (gdata.data && gdata.data.net && gdata.data.net.interfaces) {
			const currentTime = Date.now();
			// remove interfaces with 'veth' in name
			let interfaces = gdata.data.net.interfaces;
			if (!showAllInterfaces)
				interfaces = interfaces.filter(
					(iface) =>
						!iface.name.includes('veth') &&
						!iface.name.includes('docker') &&
						!iface.name.includes('br-') &&
						!iface.name.includes('lo') &&
						!iface.name.includes('tun') &&
						!iface.name.includes('wg') &&
						!iface.name.includes('vnet')
				);
			interfaces = interfaces.map((iface) => {
				const prev = previousNetworkStats[iface.name] || {
					received: iface.rx,
					transmitted: iface.tx,
					time: currentTime - 1000
				};

				// Calculate bytes per second
				const timeDiff = Math.max(0.1, (currentTime - prev.time) / 1000);
				const receivedRate = Math.max(0, (iface.rx - prev.received) / timeDiff);
				const transmittedRate = Math.max(0, (iface.tx - prev.transmitted) / timeDiff);

				// Update previous stats
				previousNetworkStats[iface.name] = {
					received: iface.rx,
					transmitted: iface.tx,
					time: currentTime
				};

				// Store interface with rates
				networkInterfaces[iface.name] = {
					name: iface.name,
					receiveRate: receivedRate,
					transmitRate: transmittedRate
				};

				return {
					...iface,
					receivedRate,
					transmittedRate
				};
			});

			// Sort interfaces by name
			sortedInterfaces = [...interfaces].sort((a, b) => a.name.localeCompare(b.name));
		}
	});

	// Handle interface selection
	function handleInterfaceSelect(e: any) {
		selectedInterface = e.target.value;
	}
</script>

{#if gdata.data}
	<div class="card">
		<p class="card-title">Network Interfaces</p>
		<div class="chart-controls">
			<label>
				<span>Show All Interfaces</span>
				<label class="switch">
					<input type="checkbox" bind:checked={showAllInterfaces} />
					<span class="slider"></span>
				</label>
			</label>
		</div>
		<div id="network-interfaces">
			{#if sortedInterfaces.length > 0}
				{#each sortedInterfaces as iface}
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
								<span class="info-value">{formatBytesPerSecond(iface.receivedRate)}</span>
							</div>
							<div class="info-item">
								<span class="info-label">Transmit Rate:</span>
								<span class="info-value">{formatBytesPerSecond(iface.transmittedRate)}</span>
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
{/if}
