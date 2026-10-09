<script lang="ts">
	import chartjs, { type ChartConfiguration } from 'chart.js/auto';
	import { applyChartJsColors, getChartColors } from '$lib/theme.svelte';

	let { timestamps, data, labels, colors, bg_colors, fills, yAxisLabel, autoScale } = $props();

	let chartCanvas: HTMLCanvasElement;
	var chart: chartjs;
	const maxDataPoints = 60;

	let timestamps_padded = $derived(
		Array(maxDataPoints - timestamps.length)
			.fill(0)
			.concat(timestamps)
	);

	let initialized = false;
	const getDataset = (label: string, color: string, bg_color: string, fill: boolean) => ({
		label,
		data: [],
		borderColor: color,
		tension: 0.3,
		fill: fill,
		backgroundColor: bg_color
	});

	let datasets = $derived.by(() => {
		let nextDatasets = [];
		for (let i = 0; i < data.length; i++) {
			nextDatasets.push(getDataset(labels[i], colors[i], bg_colors[i], fills[i]));
		}
		return nextDatasets;
	});

	let chartData = $derived.by(() => ({
		labels: Array(maxDataPoints).fill(''),
		datasets: datasets
	}));

	let chartColors = $derived(getChartColors());

	let chartConfig = $derived.by(
		() =>
			({
				type: 'line',
				data: chartData,
				options: {
					responsive: true,
					maintainAspectRatio: false,
					animation: false,
					scales: {
						y: {
							beginAtZero: true,
							title: {
								display: true,
								text: yAxisLabel,
								color: chartColors.text
							},
							grid: {
								color: chartColors.grid
							},
							ticks: {
								color: chartColors.text,
								autoSkip: true
							},
							suggestedMin: autoScale ? undefined : 0,
							suggestedMax: autoScale ? undefined : 100
						},
						x: {
							grid: {
								color: chartColors.grid
							},
							ticks: {
								color: chartColors.text,
								callback: function (_value: unknown, index: number) {
									if (timestamps_padded[index] === 0) return '';
									return (timestamps_padded[index] - Date.now() / 1000).toFixed(0) + 's';
								}
							}
						}
					},
					plugins: {
						legend: {
							labels: { color: chartColors.text }
						}
					}
				}
			}) as ChartConfiguration
	);

	$effect(() => {
		if (data.length > 0) {
			// pre pad all arays to maxDataPoints
			let data_padded = [];
			for (let i = 0; i < data.length; i++) {
				data_padded.push(
					Array(maxDataPoints - data[i].length)
						.fill(0)
						.concat(data[i])
				);
			}

			if (!initialized) {
				initialized = true;
				chart = new chartjs(chartCanvas, chartConfig);
			} else {
				chart.data.datasets[0].data = data;
				data_padded.forEach((_core: unknown, i: number) => {
					chart.data.datasets[i].data = data_padded[i];
				});
				chart.update();
			}
		}
	});

	// Recolor the existing chart when the theme changes
	$effect(() => {
		const colors = chartColors;
		if (chart) applyChartJsColors(chart, colors);
	});
</script>

<canvas bind:this={chartCanvas}></canvas>
