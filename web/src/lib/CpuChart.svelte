<script lang="ts">
	import chartjs, { type ChartConfiguration } from 'chart.js/auto';

	let { data, name, yAxisLabel, autoScale, cores, show_avg, show_cores } = $props();

	let chartCanvas: HTMLCanvasElement;
	var chart: chartjs;
	const maxDataPoints = 60;

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

	const timestamps = Array(maxDataPoints).fill(new Date());
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
						// format: 'HH:mm:ss',
						callback: function (_: any, index: number) {
							return index - maxDataPoints <= 0 ? 2 * (index - maxDataPoints) + 's' : '';
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

	function transpose(a: number[][]) {
		if (a.length === 0) return [];

		return a[0].map((_, colIndex) => a.map((row) => row[colIndex]));
	}
	let tcores = $derived(transpose(cores));
	$effect(() => {
		if (data.length > 0) {
			// pre pad all arays to maxDataPoints
			let data_padded = Array(maxDataPoints - data.length).fill(0).concat(data);
			let cores_padded = tcores.map((core: any) => Array(maxDataPoints - core.length).fill(0).concat(core));

			if (!initialized) {
				initialized = true;

				cores_padded.forEach((_: any, i: number) => {
					const color = `hsl(${(i * 360) / cores_padded.length + 200}, 70%, 60%)`;
					chartData.datasets.push(getDataset(`Core ${i + 1}`, color, '#222', false));
				});

				chart = new chartjs(chartCanvas, chartConfig);
			} else {
				chart.data.datasets[0].data = data_padded;
				cores_padded.forEach((core: any, i: number) => {
					chart.data.datasets[i + 1].data = core;
					chart.data.datasets[i + 1].hidden = !show_cores;
				});

				chart.data.datasets[0].hidden = !show_avg;

				chart.update();
			}
		}
	});
</script>

<canvas bind:this={chartCanvas}></canvas>
