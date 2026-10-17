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

const BYTE_UNITS = ['B', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB'];

// Format bytes to human readable format, e.g. "1.0 GiB"
export function formatBytes(bytes: number, decimals: number = 1) {
	bytes = Math.floor(bytes);
	if (!(bytes > 0)) return '0 B';

	const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), BYTE_UNITS.length - 1);
	if (i === 0) return `${bytes} B`;

	// Fixed precision so values line up and don't change width as they update
	return `${(bytes / 1024 ** i).toFixed(Math.max(0, decimals))} ${BYTE_UNITS[i]}`;
}

// Format a duration in seconds, e.g. "3d 4h 12m"
export function formatUptime(seconds: number): string {
	const days = Math.floor(seconds / 86400);
	const hours = Math.floor((seconds % 86400) / 3600);
	const minutes = Math.floor((seconds % 3600) / 60);
	if (days > 0) return `${days}d ${hours}h ${minutes}m`;
	if (hours > 0) return `${hours}h ${minutes}m`;
	return `${minutes}m ${Math.floor(seconds % 60)}s`;
}

// Format bytes per second with appropriate unit
export function formatBytesPerSecond(bytesPerSecond: number, decimals: number = 1) {
	return `${formatBytes(bytesPerSecond, decimals)}/s`;
}

// Webhook headers are edited as one "Name: value" per line. Only the first colon
// separates, so values such as URLs keep theirs.
export function parseHeaders(text: string) {
	const headers: Record<string, string> = {};
	for (const line of text.split('\n')) {
		const colon = line.indexOf(':');
		if (colon <= 0) continue;
		const key = line.slice(0, colon).trim();
		const value = line.slice(colon + 1).trim();
		if (key && value) headers[key] = value;
	}
	return headers;
}

export function formatHeaders(headers: Record<string, string>) {
	return Object.entries(headers)
		.map(([key, value]) => `${key}: ${value}`)
		.join('\n');
}

// Loopback and container/VM plumbing, hidden unless the user asks to see every interface
const VIRTUAL_INTERFACE_PREFIXES = ['veth', 'docker', 'br-', 'vnet'];

export function isVirtualInterface(name: string) {
	return name === 'lo' || VIRTUAL_INTERFACE_PREFIXES.some((prefix) => name.startsWith(prefix));
}
