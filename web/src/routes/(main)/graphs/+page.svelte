<script lang="ts">
	import { onMount } from 'svelte';
	import { createChart, AreaSeries, ColorType } from 'lightweight-charts';
	import { formatBytes, formatBytesPerSecond, types2names } from '$lib/utils.svelte';
	import { getHistoricalData } from '$lib/api';
	import type { HistoricalSeries } from '$lib/types';
	let resolution = $state('minute');
	let timespan = $state(3600);
	let showAll = $state(false); // Add state for showing less important charts
	let isLoading = $state(true); // Add loading state
	let connectionError = $state(false); // Add connection error state

	const resolutionOptions = [
		{ value: 'minute', label: 'Minute' },
		{ value: 'hour', label: 'Hour' },
		{ value: 'day', label: 'Day' }
	];

	const timespanOptions = [
		{ value: 3600, label: 'Last Hour' },
		{ value: 21600, label: 'Last 6 Hours' },
		{ value: 86400, label: 'Last 24 Hours' },
		{ value: 604800, label: 'Last 7 Days' },
		{ value: 2592000, label: 'Last 30 Days' },
		{ value: 7776000, label: 'Last 90 Days' },
		{ value: 31536000, label: 'Last 365 Days' }
	];

	// Get available timespan options based on current resolution
	function getAvailableTimespanOptions() {
		if (resolution === 'hour') {
			return timespanOptions.filter((option) => option.value >= 21600);
		} else if (resolution === 'day') {
			return timespanOptions.filter((option) => option.value >= 604800);
		} else if (resolution === 'minute') {
			return timespanOptions.filter((option) => option.value <= 2592000);
		} else {
			return timespanOptions;
		}
	}

	function isNegligible(series: HistoricalSeries): boolean {
		// Check name patterns
		if (
			(series.name.includes('overlay') && series.cat == 'disk') ||
			(series.name.includes('/boot') && series.cat == 'disk') ||
			(series.name.includes('veth') && series.cat == 'net') ||
			(series.name.includes('br-') && series.cat == 'net') ||
			(series.name.includes('vnet') && series.cat == 'net') ||
			(series.name.includes('docker') && series.cat == 'net') ||
			(series.name.includes('tun') && series.cat == 'net') ||
			(series.name.includes('wg') && series.cat == 'net') ||
			(series.name == 'lo' && series.cat == 'net')
		) {
			return true;
		}

		// Check series types
		if (['total_read', 'total_write', 'rx', 'tx'].includes(series.stype)) {
			return true;
		}

		return false;
	}

	let seriesData: HistoricalSeries[] = $state([]);
	let chartContainers: HTMLDivElement[] = $state([]);
	let charts: any[] = [];
	let areaSeries: any[] = [];

	const timeOffset = new Date().getTimezoneOffset() * 60;

	// Define the category order
	const categoryOrder = ['general', 'net', 'disk'];
	const typeOrder = [
		'cpu_usage',
		'mem_usage',
		'swap_usage',
		'load_avg_1',
		'load_avg_5',
		'load_avg_15',
		'rx_rate',
		'tx_rate',
		'read_rate',
		'write_rate',
		'total_read',
		'total_write',
		'rx',
		'tx'
	];

	// Format underscore separated names to readable format
	function formatName(name: string, stype: string): string {
		let t0 = types2names[stype] || stype;
		if (name === 'system') {
			return t0;
		}
		return `${t0} - ${name}`;
	}

	function getPriceFormat(stype: string) {
		if (['swap_usage', 'cpu_usage', 'mem_usage', 'disk_usage'].includes(stype)) {
			return {
				type: 'percent' as const,
				precision: 2
			};
		} else if (
			[
				'rx',
				'tx',
				'rx_rate',
				'tx_rate',
				'total_read',
				'total_write',
				'read_rate',
				'write_rate'
			].includes(stype)
		) {
			return {
				type: 'volume' as const
			};
		} else {
			return {
				type: 'price' as const,
				precision: 2
			};
		}
	}

	async function fetchHistoricalData(): Promise<HistoricalSeries[]> {
		isLoading = true;
		connectionError = false; // Reset connection error state

		try {
			const start_time = Math.floor(Date.now() / 1000 - timespan);
			const result = await getHistoricalData({
				resolution: resolution as 'second' | 'minute' | 'hour' | 'day',
				start_time
			});

			if (!result.success) {
				throw new Error(result.error);
			}

			const data = result.data;

			// Filter out less important series if showAll is false
			let filteredData = data;
			if (!showAll) {
				filteredData = data.filter((series: HistoricalSeries) => !isNegligible(series));
			}

			return filteredData.sort((a: HistoricalSeries, b: HistoricalSeries) => {
				const indexA = categoryOrder.indexOf(a.cat);
				const indexB = categoryOrder.indexOf(b.cat);
				const typeIndexA = typeOrder.indexOf(a.stype);
				const typeIndexB = typeOrder.indexOf(b.stype);
				return (
					(indexA === -1 ? 999 : indexA) - (indexB === -1 ? 999 : indexB) ||
					(typeIndexA === -1 ? 999 : typeIndexA) - (typeIndexB === -1 ? 999 : typeIndexB) ||
					a.name.localeCompare(b.name)
				);
			});
		} catch (error) {
			console.error('Failed to fetch historical data:', error);
			connectionError = true; // Set connection error state to true
			return [];
		} finally {
			isLoading = false;
		}
	}

	function formatChartData(series: HistoricalSeries) {
		return series.timestamps.map((timestamp, index) => ({
			time: (timestamp - timeOffset) as unknown as import('lightweight-charts').Time,
			value: series.values[index]
		}));
	}

	function getColorForSeries(index: number) {
		const colors = [
			{ line: '#2196F3', top: '#2196F3', bottom: 'rgba(33, 150, 243, 0.28)' },
			{ line: '#FF4081', top: '#FF4081', bottom: 'rgba(255, 64, 129, 0.28)' },
			{ line: '#4CAF50', top: '#4CAF50', bottom: 'rgba(76, 175, 80, 0.28)' },
			{ line: '#FF9800', top: '#FF9800', bottom: 'rgba(255, 152, 0, 0.28)' },
			{ line: '#9C27B0', top: '#9C27B0', bottom: 'rgba(156, 39, 176, 0.28)' }
		];
		return colors[index % colors.length];
	}

	function initCharts() {
		charts.forEach((chart) => chart.remove());
		charts = [];
		areaSeries = [];

		chartContainers.forEach((container, i) => {
			if (!container || i >= seriesData.length) return;

			const chartOptions = {
				autosize: true,
				layout: {
					textColor: '#d1d4dc',
					background: { type: ColorType.Solid, color: '#171721' }
				},
				grid: {
					vertLines: { color: '#2B2B43' },
					horzLines: { color: '#2B2B43' }
				},
				timeScale: {
					timeVisible: true,
					secondsVisible: true
				}
			};

			const chart = createChart(container, chartOptions);
			if (
				seriesData[i].cat === 'net' ||
				(seriesData[i].cat === 'disk' && seriesData[i].stype !== 'disk_usage')
			) {
				if (seriesData[i].stype.toLowerCase().includes('rate')) {
					chart.applyOptions({
						localization: {
							priceFormatter: formatBytesPerSecond
						}
					});
				} else {
					chart.applyOptions({
						localization: {
							priceFormatter: formatBytes
						}
					});
				}
			}

			const color = getColorForSeries(i);
			const currentSeries = seriesData[i];

			const seriesDataseries = chart.addSeries(AreaSeries, {
				lineColor: color.line,
				topColor: color.top,
				bottomColor: color.bottom,
				priceFormat: getPriceFormat(currentSeries.stype)
			});

			seriesDataseries.setData(formatChartData(currentSeries));
			chart.timeScale().fitContent();

			charts.push(chart);
			areaSeries.push(seriesDataseries);
		});
	}

	async function refreshCharts() {
		if (resolution === 'hour' && timespan < 21600) {
			timespan = 21600;
		}
		if (resolution === 'day' && timespan < 604800) {
			timespan = 604800;
		}
		if (resolution === 'minute' && timespan > 2592000) {
			timespan = 2592000;
		}
		seriesData = [];
		seriesData = await fetchHistoricalData();
		if (seriesData.length > 0) {
			initCharts();
		}
	}

	// Run when the component is mounted
	onMount(async () => {
		seriesData = await fetchHistoricalData();
	});

	$effect(() => {
		initCharts();
	});
</script>

<div class="container">
	<div class="controls">
		<div class="control-group">
			<label for="resolution">Resolution:</label>
			<select id="resolution" bind:value={resolution} onchange={refreshCharts}>
				{#each resolutionOptions as option (option.value)}
					<option value={option.value}>{option.label}</option>
				{/each}
			</select>
		</div>

		<div class="control-group">
			<label for="timespan">Timespan:</label>
			<select id="timespan" bind:value={timespan} onchange={refreshCharts}>
				{#each getAvailableTimespanOptions() as option (option.value)}
					<option value={option.value}>{option.label}</option>
				{/each}
			</select>
		</div>

		<label>
			<span>Show All</span>
			<label class="switch">
				<input type="checkbox" bind:checked={showAll} onchange={refreshCharts} />
				<span class="slider"></span>
			</label>
		</label>
	</div>

	{#if isLoading}
		<div class="loading">
			<div class="spinner"></div>
			<p>Loading chart data...</p>
		</div>
	{:else if connectionError}
		<div class="error-container">
			<p>Connection failed</p>
			<p>Could not connect to the data service. Please check your network connection.</p>
			<button onclick={refreshCharts} class="retry-button">Retry</button>
		</div>
	{:else if seriesData.length === 0}
		<div class="no-data">
			<p>No data available for the selected time period.</p>
			<p>Try adjusting your filters.</p>
		</div>
	{:else}
		{#each seriesData as series, i (series.name + series.stype)}
			<div class="chart-container">
				<h2>{formatName(series.name, series.stype)}</h2>
				<div class="chart" bind:this={chartContainers[i]}></div>
			</div>
		{/each}
	{/if}
</div>

<style>
	.container {
		width: 100%;
		background: transparent;
		padding: 20px;
	}

	.controls {
		display: flex;
		gap: 20px;
		margin-bottom: 20px;
		color: #d1d4dc;
		align-items: center;
		flex-wrap: wrap;
	}

	.control-group {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	select {
		background: #2a2e39;
		color: #d1d4dc;
		border: 1px solid #4a4e59;
		padding: 6px 12px;
		border-radius: 4px;
		font-size: 14px;
	}

	.chart-container {
		margin-bottom: 30px;
	}

	h2 {
		color: #d1d4dc;
		margin: 0 0 10px 0;
		font-size: 18px;
	}

	.chart {
		width: 100%;
		height: 250px;
		background: transparent;
	}
</style>
