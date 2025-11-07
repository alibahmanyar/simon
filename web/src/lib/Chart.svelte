<script lang="ts">
	import chartjs, { type ChartConfiguration } from 'chart.js/auto';

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

	let datasets = [];
	for (let i = 0; i < data.length; i++) {
		datasets.push(getDataset(labels[i], colors[i], bg_colors[i], fills[i]));
	}

	const chartData = {
		labels: Array(maxDataPoints).fill(''),
		datasets: datasets
	};

	const chartConfig = {
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
						color: '#e1e1e3'
					},
					grid: {
						color: 'rgba(255, 255, 255, 0.1)'
					},
					ticks: {
						color: '#e1e1e3',
						autoSkip: true
					},
					suggestedMin: autoScale ? undefined : 0,
					suggestedMax: autoScale ? undefined : 100
				},
				x: {
					grid: {
						color: 'rgba(255, 255, 255, 0.1)'
					},
					ticks: {
						color: '#e1e1e3',
						callback: function (_value: unknown, index: number) {
							if (timestamps_padded[index] === 0) return '';
							return (timestamps_padded[index] - Date.now() / 1000).toFixed(0) + 's';
						}
					}
				}
			},
			plugins: {
				legend: {
					labels: { color: '#e1e1e3' }
				}
			}
		}
	} as ChartConfiguration;

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
</script>

<canvas bind:this={chartCanvas}></canvas>
