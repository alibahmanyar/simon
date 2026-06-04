<script lang="ts">
	import { slide } from 'svelte/transition';
	import { formatUptime, formatBytes, capabilities } from '$lib/utils.svelte';
	import { gdata } from '$lib/general_socket.svelte';
	import Chart from '$lib/Chart.svelte';
	import CpuChart from '$lib/CpuChart.svelte';

	// $inspect(gdata).with(console.trace);

	let sysData = $derived(gdata.data?.current);
	let prevDataPoints = $derived(gdata.data?.prevDataPoints ?? []);

	let memoryPercentage = $derived(
		sysData
			? sysData.mem.total_mem > 0
				? ((sysData.mem.used_mem / sysData.mem.total_mem) * 100).toFixed(1)
				: 0
			: null
	);
	let swapPercentage = $derived(
		sysData
			? sysData.mem.total_swap > 0
				? ((sysData.mem.used_swap / sysData.mem.total_swap) * 100).toFixed(1)
				: 0
			: null
	);

	let timestamps = $derived(prevDataPoints.map((d) => d.t));
	let cpu_data = $derived(prevDataPoints.map((d) => d.cpu.avg_usage.toFixed(1)));
	let cores = $derived(prevDataPoints.map((d) => d.cpu.usage));
	let sys_load_1 = $derived(prevDataPoints.map((d) => d.sys.load_avg[0].toFixed(2)));
	let sys_load_5 = $derived(prevDataPoints.map((d) => d.sys.load_avg[1].toFixed(2)));
	let sys_load_15 = $derived(prevDataPoints.map((d) => d.sys.load_avg[2].toFixed(2)));
	let show_avg = $state(true),
		show_graph_cores = $state(false),
		show_cores = $state(false);
</script>

{#if sysData}
	<!-- System Overview Section -->
	<div class="two-columns">
		<!-- Left Column -->
		<div>
			<!-- CPU Overview -->
			{#if capabilities.cpu}
				<div class="card">
					<p class="card-title">CPU Usage</p>
					<span class="usage">{sysData.cpu.avg_usage.toFixed(1)}%</span>
					<div class="bar">
						<div class="bar-fill" style="width: {sysData.cpu.avg_usage.toFixed(1)}%"></div>
					</div>
					<div class="switch-row">
						<span class="switch-text">Show Cores</span>
						<label class="switch">
							<input
								type="checkbox"
								id="show-cores-toggle"
								bind:checked={show_cores}
								aria-label="Show cores"
							/>
							<span class="slider"></span>
						</label>
					</div>
					{#if show_cores}
						<div id="cores" class="core-grid" transition:slide>
							{#each sysData.cpu.usage as usage, i (i)}
								<div class="core-item">
									<p>Core {i + 1}</p>
									<span class="core-value">{usage.toFixed(1)}%</span>
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
						<span id="host-name" class="info-value">{sysData.sys.host_name}</span>
					</div>
					<div class="info-item">
						<span class="info-label">OS:</span>
						<span id="os-version" class="info-value">{sysData.sys.os_name}</span>
					</div>
					<div class="info-item">
						<span class="info-label">Kernel:</span>
						<span id="kernel-version" class="info-value">{sysData.sys.kernel_ver}</span>
					</div>
				</div>
				<div class="uptime-value" id="system-uptime">
					Uptime: {formatUptime(sysData.sys.uptime)}
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
								>{formatBytes(sysData.mem.used_mem)}/{formatBytes(sysData.mem.total_mem)}</span
							>
						</div>
					</div>
					{#if capabilities.swap}
						<div class="spacer-sm"></div>
						<p class="card-title">Swap Usage</p>
						<span class="usage">{swapPercentage}%</span>
						<div class="bar">
							<div class="bar-fill" style="width: {swapPercentage}%"></div>
						</div>
						<div class="info-grid-0">
							<div class="info-item">
								<span class="info-label">Swap Used/Total:</span>
								<span id="swap-used" class="info-value"
									>{formatBytes(sysData.mem.used_swap)}/{formatBytes(sysData.mem.total_swap)}</span
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
							<span class="load-value" id="load-1">{sysData.sys.load_avg[0].toFixed(2)}</span>
						</div>
						<div class="load-item">
							<span class="load-label">5 Minutes</span>
							<span class="load-value" id="load-5">{sysData.sys.load_avg[1].toFixed(2)}</span>
						</div>
						<div class="load-item">
							<span class="load-label">15 Minutes</span>
							<span class="load-value" id="load-15">{sysData.sys.load_avg[2].toFixed(2)}</span>
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
				<div class="switch-row">
					<span class="switch-text">Show System Average</span>
					<label class="switch">
						<input
							type="checkbox"
							id="avg-toggle"
							bind:checked={show_avg}
							aria-label="Show system average"
						/>
						<span class="slider"></span>
					</label>
				</div>
				<div class="switch-row">
					<span class="switch-text">Show All Cores</span>
					<label class="switch">
						<input
							type="checkbox"
							id="show-all-cores-toggle"
							bind:checked={show_graph_cores}
							aria-label="Show all cores"
						/>
						<span class="slider"></span>
					</label>
				</div>
			</div>
			<div class="chart-shell">
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
			<div class="chart-shell">
				<Chart
					{timestamps}
					yAxisLabel="Memory Usage (%)"
					autoScale={false}
					data={[
						prevDataPoints.map((x) => ((x.mem.used_mem * 100) / x.mem.total_mem).toFixed(2)),
						prevDataPoints.map((x) => ((x.mem.used_swap * 100) / x.mem.total_swap).toFixed(2))
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
			<div class="chart-shell chart-shell--compact">
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
