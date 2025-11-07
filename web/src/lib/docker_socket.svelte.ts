// place files you want to import through the `$lib` alias in this folder.
import { ws_url } from './utils.svelte';
import { wsStatus, type DockerInfo } from './types';

export const ddata = $state({
	data: {} as DockerInfo,
	status: wsStatus.INIT as wsStatus
});

let d_ws: WebSocket | null = null;
let reconnectAttempt = 0;
let reconnectTimeout: number | null = null;
let isReconnecting = false;

function getReconnectDelay(): number {
	// Backoff with a maximum of 10 seconds
	return Math.min(2000 * (reconnectAttempt + 1), 10000);
}

function scheduleReconnect() {
	if (isReconnecting) return;

	if (ddata.data === null) return; // Don't flood the log with errors if docker is inaccessible

	isReconnecting = true;
	const delay = getReconnectDelay();

	//console.log(`Scheduling reconnect attempt in ${delay}ms`);

	if (reconnectTimeout !== null) {
		clearTimeout(reconnectTimeout);
	}

	reconnectTimeout = setTimeout(() => {
		reconnectAttempt++;
		isReconnecting = false;
		open_ws();
	}, delay) as unknown as number;
}

export function close_ws() {
	if (d_ws !== null) {
		d_ws.close();
	}
}

export function open_ws() {
	// Clean up existing connection if any
	if (d_ws !== null) {
		try {
			d_ws.onopen = null;
			d_ws.onclose = null;
			d_ws.onerror = null;
			d_ws.onmessage = null;
			d_ws.close();
		} catch (e) {
			console.error('Error while closing existing WebSocket:', e);
		}
		d_ws = null;
	}

	try {
		d_ws = new WebSocket(import.meta.env.PROD ? ws_url('ws/d') : 'ws://localhost:30000/ws/d');
	} catch (e) {
		console.error('WebSocket connection failed: ', e);
		ddata.status = wsStatus.ERROR;
		scheduleReconnect();
		return;
	}

	d_ws.onopen = function () {
		ddata.status = wsStatus.WAITING;
		//console.log("WebSocket opened:", event);
		reconnectAttempt = 0;
	};

	d_ws.onerror = function () {
		ddata.status = wsStatus.ERROR;
		console.error('WebSocket error observed');
		// We'll let onclose handle the reconnection
	};

	d_ws.onclose = function () {
		ddata.status = wsStatus.DISCONNECTED;
		//console.log("WebSocket closed:", event);
		scheduleReconnect();
	};

	d_ws.onmessage = async function (event) {
		ddata.status = wsStatus.CONNECTED;
		const compressedData = await event.data.arrayBuffer();
		const decompressStream = new DecompressionStream('gzip');
		const decompressedStream = new ReadableStream({
			start(controller) {
				controller.enqueue(compressedData);
				controller.close();
			}
		}).pipeThrough(decompressStream);

		const responseData = await new Response(decompressedStream).text();

		ddata.data = JSON.parse(responseData) as DockerInfo;
	};
}
