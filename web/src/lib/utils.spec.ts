import { describe, it, expect } from 'vitest';
import {
	formatBytes,
	formatBytesPerSecond,
	formatHeaders,
	formatUptime,
	isVirtualInterface,
	parseHeaders
} from './utils.svelte';

describe('formatBytes', () => {
	it('uses B for whole bytes and keeps a fixed precision above that', () => {
		expect(formatBytes(0)).toBe('0 B');
		expect(formatBytes(512)).toBe('512 B');
		expect(formatBytes(1024)).toBe('1.0 KiB');
		expect(formatBytes(1024 ** 3)).toBe('1.0 GiB');
		expect(formatBytes(5.7 * 1024 ** 3)).toBe('5.7 GiB');
	});

	it('treats negative and missing values as zero', () => {
		expect(formatBytes(-5)).toBe('0 B');
		expect(formatBytes(NaN)).toBe('0 B');
	});

	it('stops at the largest unit', () => {
		expect(formatBytes(1024 ** 6)).toBe('1024.0 PiB');
	});

	it('formats rates with the same units', () => {
		expect(formatBytesPerSecond(0)).toBe('0 B/s');
		expect(formatBytesPerSecond(10 * 1024 ** 2)).toBe('10.0 MiB/s');
	});
});

describe('formatUptime', () => {
	it('reads as a duration rather than a time of day', () => {
		expect(formatUptime(42)).toBe('0m 42s');
		expect(formatUptime(4 * 3600 + 54 * 60 + 8)).toBe('4h 54m');
		expect(formatUptime(3 * 86400 + 4 * 3600 + 12 * 60)).toBe('3d 4h 12m');
	});
});

describe('isVirtualInterface', () => {
	it('hides loopback and container plumbing', () => {
		for (const name of ['lo', 'veth3f2a1b', 'docker0', 'br-c371af850a4e', 'vnet0']) {
			expect(isVirtualInterface(name)).toBe(true);
		}
	});

	it('keeps real interfaces whose names merely contain those strings', () => {
		for (const name of ['wlo1', 'eth0', 'enp8s0f3u1', 'br0', 'wg0', 'tun0']) {
			expect(isVirtualInterface(name)).toBe(false);
		}
	});
});

describe('parseHeaders', () => {
	it('splits each line on the first colon only', () => {
		expect(parseHeaders('Click: https://example.com:8443/x\nTitle: Disk: almost full')).toEqual({
			Click: 'https://example.com:8443/x',
			Title: 'Disk: almost full'
		});
	});

	it('skips blank, valueless and nameless lines', () => {
		expect(parseHeaders('\nX-Empty:\n: no-name\nContent-Type: application/json\n')).toEqual({
			'Content-Type': 'application/json'
		});
	});

	it('round-trips with formatHeaders', () => {
		const headers = { 'Content-Type': 'application/json', 'Access-Token': 'o.abc:def' };
		expect(parseHeaders(formatHeaders(headers))).toEqual(headers);
	});
});
