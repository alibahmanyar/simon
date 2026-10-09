<script lang="ts">
	import chartjs, { type ChartConfiguration } from 'chart.js/auto';
	import { applyChartJsColors, getChartColors } from '$lib/theme.svelte';

	let { timestamps, data, yAxisLabel, autoScale, cores, show_avg, show_cores } = $props();

	let chartCanvas: HTMLCanvasElement;
	var chart: chartjs;
	const maxDataPoints = 60;

	let timestamps_padded = $derived(
		Array(maxDataPoints - timestamps.length)
			.fill(0)
			.concat(timestamps)
	);
	let tcores = $derived(transpose(cores));

	let initialized = false;
	const getDataset = (label: string, color: string, bg_color: string, fill: boolean) => ({
		label,
		data: [],
		borderColor: color,
		tension: 0.3,
		fill: fill,
		backgroundColor: bg_color,
		hidden: label !== 'System Average'
	});

	const chartData = {
		labels: Array(maxDataPoints).fill(''),
		datasets: [getDataset('System Average', '#4ade80', '#4ade8033', true)]
	};

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

	function transpose(a: number[][]) {
		if (a.length === 0) return [];

		return a[0].map((_, colIndex) => a.map((row) => row[colIndex]));
	}

	$effect(() => {
		if (data.length > 0) {
			// pre pad all arays to maxDataPoints
			let data_padded = Array(maxDataPoints - data.length)
				.fill(0)
				.concat(data);
			let cores_padded = tcores.map((core: number[]) =>
				Array(maxDataPoints - core.length)
					.fill(0)
					.concat(core)
			);

			if (!initialized) {
				initialized = true;

				cores_padded.forEach((_core: unknown, i: number) => {
					const color = `hsl(${(i * 360) / cores_padded.length + 200}, 70%, 60%)`;
					chartData.datasets.push(getDataset(`Core ${i + 1}`, color, 'transparent', false));
				});

				chart = new chartjs(chartCanvas, chartConfig);
			} else {
				chart.data.datasets[0].data = data_padded;
				cores_padded.forEach((core: number[], i: number) => {
					chart.data.datasets[i + 1].data = core;
					chart.data.datasets[i + 1].hidden = !show_cores;
				});

				chart.data.datasets[0].hidden = !show_avg;

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
