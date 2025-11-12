import { page } from '$app/state';
import type { SystemCapabilities } from './types';
import { getSystemCapabilities } from './api';

export const capabilities = $state<SystemCapabilities>({
	cpu: false,
	memory: false,
	swap: false,
	disk: false,
	network: false,
	load_average: false,
	processes: false,
	docker: false,
	file_serving: false
});

export async function updateCapabilities() {
	const data = await getSystemCapabilities();
	if (data.success) {
		Object.assign(capabilities, data.data);
	}
	return data;
}

export const types2names: { [key: string]: string } = {
	cpu_usage: 'CPU Usage',
	mem_usage: 'Memory Usage',
	swap_usage: 'Swap Usage',
	load_avg_1: '1 min Load Average',
	load_avg_5: '5 min Load Average',
	load_avg_15: '15 min Load Average',
	rx_rate: 'RX Rate',
	tx_rate: 'TX Rate',
	read_rate: 'Read Rate',
	write_rate: 'Write Rate',
	total_read: 'Total Read',
	total_write: 'Total Write',
	disk_usage: 'Disk Usage',
	rx: 'RX',
	tx: 'TX'
};

export const cat2names: { [key: string]: string } = {
	sys: 'System',
	net: 'Network',
	disk: 'Storage'
};

export const var2unit: { [key: string]: string } = {
	cpu_usage: '%',
	mem_usage: '%',
	swap_usage: '%',
	load_avg_1: '',
	load_avg_5: '',
	load_avg_15: '',
	rx_rate: 'B/s',
	tx_rate: 'B/s',
	read_rate: 'B/s',
	write_rate: 'B/s',
	total_read: 'B',
	total_write: 'B',
	disk_usage: '%',
	rx: 'B',
	tx: 'B'
};

const paths = [
	'/',
	'/alerts',
	'/docker',
	'/files',
	'/graphs',
	'/network',
	'/notif_methods',
	'/processes',
	'/storage'
];

function getBaseUrl() {
	// In development, use localhost with the dev server port
	if (import.meta.env.DEV) {
		return 'http://localhost:30000';
	}

	// In production, extract base URL from current page
	let href = page.url.href;
	if (href[href.length - 1] === '/') {
		href = href.slice(0, -1);
	}
	for (const path of paths) {
		if (href.endsWith(path)) {
			return href.slice(0, -path.length);
		}
	}
	return href;
}

export function ws_url(s: string) {
	return getBaseUrl().replaceAll('http', 'ws') + '/' + s;
}

export function url(s: string) {
	return getBaseUrl() + '/' + s;
}

// Format bytes to human readable format
export function formatBytes(bytes: number, decimals: number = 1) {
	bytes = Math.floor(bytes);
	if (bytes <= 0) return '0 Bytes';

	const k = 1024;
	const dm = decimals < 0 ? 0 : decimals;
	const sizes = ['Bytes', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB'];

	const i = Math.floor(Math.log(bytes) / Math.log(k));

	return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

export function formatUptime(seconds: number): string {
	const days = Math.floor(seconds / 86400);
	seconds %= 86400;
	const hours = Math.floor(seconds / 3600);
	seconds %= 3600;
	const minutes = Math.floor(seconds / 60);
	seconds %= 60;
	if (days > 0) {
		return `${days}d ${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
	}
	return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
}

// Format bytes per second with appropriate unit
export function formatBytesPerSecond(bytesPerSecond: number, decimals: number = 1) {
	bytesPerSecond = Math.floor(bytesPerSecond);
	if (bytesPerSecond <= 0) return '0 B/s';

	const k = 1024;
	const dm = decimals < 0 ? 0 : decimals;
	const sizes = ['B/s', 'KiB/s', 'MiB/s', 'GiB/s', 'TiB/s'];

	const i = Math.floor(Math.log(bytesPerSecond) / Math.log(k));

	return parseFloat((bytesPerSecond / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}
