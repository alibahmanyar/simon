<script lang="ts">
	import { slide } from 'svelte/transition';
	import { formatUptime, formatBytes, capabilities } from '$lib/utils.svelte';
	import { gdata } from '$lib/general_socket.svelte';
	import Chart from '$lib/Chart.svelte';
	import CpuChart from '$lib/CpuChart.svelte';

	// $inspect(gdata).with(console.trace);

	let memoryPercentage = $derived(
		gdata.data
			? gdata.data.mem.total_mem > 0
				? ((gdata.data.mem.used_mem / gdata.data.mem.total_mem) * 100).toFixed(1)
				: 0
			: null
	);
	let swapPercentage = $derived(
		gdata.data
			? gdata.data.mem.total_swap > 0
				? ((gdata.data.mem.used_swap / gdata.data.mem.total_swap) * 100).toFixed(1)
				: 0
			: null
	);

	let timestamps = $derived(gdata.prevDataPoints.map((d) => d.t));
	let cpu_data = $derived(gdata.prevDataPoints.map((d) => d.cpu.avg_usage.toFixed(1)));
	let cores = $derived(gdata.prevDataPoints.map((d) => d.cpu.usage));
	let sys_load_1 = $derived(gdata.prevDataPoints.map((d) => d.sys.load_avg[0]));
	let sys_load_5 = $derived(gdata.prevDataPoints.map((d) => d.sys.load_avg[1]));
	let sys_load_15 = $derived(gdata.prevDataPoints.map((d) => d.sys.load_avg[2]));
	let show_avg = $state(true),
		show_graph_cores = $state(false),
		show_cores = $state(false);
</script>

{#if gdata.data}
	<!-- System Overview Section -->
	<div class="two-columns">
		<!-- Left Column -->
		<div>
			<!-- CPU Overview -->
			{#if capabilities.cpu}
				<div class="card">
					<p class="card-title">CPU Usage</p>
					<span class="usage">{gdata.data.cpu.avg_usage.toFixed(1)}%</span>
					<div class="bar">
						<div class="bar-fill" style="width: {gdata.data.cpu.avg_usage.toFixed(1)}%"></div>
					</div>
					<label class="switch-label">
						<span>Show Cores</span>
						<label class="switch">
							<input type="checkbox" id="allCoresToggle" bind:checked={show_cores} />
							<span class="slider"></span>
						</label>
					</label>
					{#if show_cores}
						<div id="cores" class="core-grid" transition:slide>
							{#each gdata.data.cpu.usage as usage, i}
								<div class="core">
									<div class="core-item">
										<p>Core {i + 1}</p>
										<span class="core-value">{usage.toFixed(1)}%</span>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			{/if}
			<!-- Quick Stats -->
			<div class="card">
				<div class="info-grid-0">
					<div class="info-item">
						<span class="info-label">Host Name:</span>
						<span id="host-name" class="info-value">{gdata.data.sys.host_name}</span>
					</div>
					<div class="info-item">
						<span class="info-label">OS:</span>
						<span id="os-version" class="info-value">{gdata.data.sys.os_name}</span>
					</div>
					<div class="info-item">
						<span class="info-label">Kernel:</span>
						<span id="kernel-version" class="info-value">{gdata.data.sys.kernel_ver}</span>
					</div>
				</div>
				<div class="uptime-value" id="system-uptime">
					Uptime: {formatUptime(gdata.data.sys.uptime)}
				</div>
			</div>
		</div>

		<!-- Right Column -->
		<div>
			<!-- Quick Memory Overview -->
			{#if capabilities.memory}
				<div class="card">
					<p class="card-title">Memory Usage</p>
					<span class="usage">{memoryPercentage}%</span>
					<div class="bar">
						<div class="bar-fill" style="width: {memoryPercentage}%"></div>
					</div>
					<div class="info-grid-0">
						<div class="info-item">
							<span class="info-label">Memory Used/Total:</span>
							<span id="memory-used" class="info-value"
								>{formatBytes(gdata.data.mem.used_mem)}/{formatBytes(
									gdata.data.mem.total_mem
								)}</span
							>
						</div>
					</div>
					{#if capabilities.swap}
						<div style="height: 2rem;"></div>
						<p class="card-title">Swap Usage</p>
						<span class="usage">{swapPercentage}%</span>
						<div class="bar">
							<div class="bar-fill" style="width: {swapPercentage}%"></div>
						</div>
						<div class="info-grid-0">
							<div class="info-item">
								<span class="info-label">Swap Used/Total:</span>
								<span id="memory-used" class="info-value"
									>{formatBytes(gdata.data.mem.used_swap)}/{formatBytes(
										gdata.data.mem.total_swap
									)}</span
								>
							</div>
						</div>
					{/if}
				</div>
			{/if}

			<!-- Load Average Card -->
			{#if capabilities.load_average}
				<div class="card">
					<p class="card-title">System Load</p>
					<div class="load-average">
						<div class="load-item">
							<span class="load-label">1 Minute</span>
							<span class="load-value" id="load-1">{gdata.data.sys.load_avg[0]}</span>
						</div>
						<div class="load-item">
							<span class="load-label">5 Minutes</span>
							<span class="load-value" id="load-5">{gdata.data.sys.load_avg[1]}</span>
						</div>
						<div class="load-item">
							<span class="load-label">15 Minutes</span>
							<span class="load-value" id="load-15">{gdata.data.sys.load_avg[2]}</span>
						</div>
					</div>
				</div>
			{/if}
		</div>
	</div>

	<!-- CPU Chart Section -->
	{#if capabilities.cpu}
		<div class="chart-card">
			<p class="card-title">CPU Usage</p>
			<div class="chart-controls">
				<label>
					<span>Show System Average</span>
					<label class="switch">
						<input type="checkbox" id="avgToggle" bind:checked={show_avg} />
						<span class="slider"></span>
					</label>
				</label>
				<label class="switch-label">
					<span>Show All Cores</span>
					<label class="switch">
						<input type="checkbox" id="allCoresToggle" bind:checked={show_graph_cores} />
						<span class="slider"></span>
					</label>
				</label>
			</div>
			<div style="min-height: 40vh;">
				<CpuChart
					{timestamps}
					autoScale={false}
					yAxisLabel="CPU Usage (%)"
					data={cpu_data}
					{cores}
					{show_avg}
					show_cores={show_graph_cores}
				/>
			</div>
		</div>
	{/if}

	<!-- Memory Chart -->
	{#if capabilities.memory}
		<div class="chart-card">
			<p class="card-title">Memory Usage</p>
			<div style="min-height: 40vh;">
				<Chart
					{timestamps}
					yAxisLabel="Memory Usage (%)"
					autoScale={false}
					data={[
						gdata.prevDataPoints.map((x) => (x.mem.used_mem * 100) / x.mem.total_mem),
						gdata.prevDataPoints.map((x) => (x.mem.used_swap * 100) / x.mem.total_swap)
					]}
					labels={['Memory', 'Swap']}
					colors={['#4dabf7', '#ae3ec9']}
					bg_colors={['rgba(77, 171, 247, 0.1)', 'rgba(174, 62, 201, 0.1)']}
					fills={[true, false]}
				/>
			</div>
		</div>
	{/if}

	<!-- Load Average Chart -->
	{#if capabilities.load_average}
		<div class="chart-card">
			<p class="card-title">System Load</p>
			<div style="min-height: 30vh;">
				<Chart
					{timestamps}
					yAxisLabel="Load Average"
					autoScale={true}
					data={[sys_load_1, sys_load_5, sys_load_15]}
					labels={['1min', '5min', '15min']}
					colors={['#4dabf7', '#ae3ec9', '#20c997']}
					bg_colors={[
						'rgba(77, 171, 247, 0.1)',
						'rgba(174, 62, 201, 0.1)',
						'rgba(32, 201, 151, 0.1)'
					]}
					fills={[false, false, false]}
				/>
			</div>
		</div>
	{/if}
{/if}
