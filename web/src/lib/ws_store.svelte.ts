/**
 * WebSocket Store Factory
 *
 * Creates a Svelte reactive store that manages a single WebSocket connection
 * with automatic reconnection (exponential backoff, max 10s).
 *
 * Usage:
 *   const mySocket = createWebSocketStore<MyDataType>('ws/endpoint', {
 *     onMessage: (data) => processedData,
 *     initialData: null,
 *   });
 *
 *   // In a component:
 *   mySocket.open();
 *   // Access reactive state: mySocket.store.data, mySocket.store.status
 *   mySocket.close();
 */
import { ws_url } from './utils.svelte';
import { wsStatus } from './types';

export interface WebSocketStoreOptions<T> {
	/** Transform raw decompressed JSON text into the desired data shape.
	 *  Receives parsed JSON by default. Override for custom processing. */
	onMessage?: (parsed: T, previous: T | null) => T;
	/** Initial data value before the first message arrives */
	initialData: T | null;
	/** If true, incoming data is gzip-compressed (default: true) */
	compressed?: boolean;
	/** Whether to skip reconnect when data is null (used by Docker socket) */
	skipReconnectWhenNull?: boolean;
}

export interface WebSocketStore<T> {
	/** Reactive state object — import and read in components */
	store: { data: T | null; status: wsStatus; lastMessageAt: number | null };
	/** Open the WebSocket connection. Safe to call multiple times. */
	open: () => void;
	/** Close the WebSocket connection and cancel any pending reconnects. */
	close: () => void;
}

export function createWebSocketStore<T>(
	endpoint: string,
	options: WebSocketStoreOptions<T>
): WebSocketStore<T> {
	const compressed = options.compressed ?? true;
	const skipReconnectWhenNull = options.skipReconnectWhenNull ?? false;

	const store = $state({
		data: options.initialData as T | null,
		status: wsStatus.INIT as wsStatus,
		/** Client time (ms) of the last message, to tell how stale the data is */
		lastMessageAt: null as number | null
	});

	let ws: WebSocket | null = null;
	let reconnectAttempt = 0;
	let reconnectTimeout: number | null = null;
	let isReconnecting = false;

	function getReconnectDelay(): number {
		return Math.min(2000 * (reconnectAttempt + 1), 10000);
	}

	function scheduleReconnect() {
		if (isReconnecting) return;
		if (skipReconnectWhenNull && store.data === null) return;

		isReconnecting = true;
		const delay = getReconnectDelay();

		if (reconnectTimeout !== null) {
			clearTimeout(reconnectTimeout);
		}

		reconnectTimeout = setTimeout(() => {
			reconnectAttempt++;
			isReconnecting = false;
			open();
		}, delay) as unknown as number;
	}

	function cleanupWs() {
		if (ws !== null) {
			try {
				ws.onopen = null;
				ws.onclose = null;
				ws.onerror = null;
				ws.onmessage = null;
				ws.close();
			} catch (e) {
				console.error(`Error while closing WebSocket (${endpoint}):`, e);
			}
			ws = null;
		}
	}

	function close() {
		if (reconnectTimeout !== null) {
			clearTimeout(reconnectTimeout);
			reconnectTimeout = null;
		}
		isReconnecting = false;
		cleanupWs();
	}

	function open() {
		cleanupWs();

		try {
			ws = new WebSocket(
				import.meta.env.PROD ? ws_url(endpoint) : `ws://localhost:30000/${endpoint}`
			);
		} catch (e) {
			console.error(`WebSocket connection failed (${endpoint}):`, e);
			store.status = wsStatus.ERROR;
			scheduleReconnect();
			return;
		}

		ws.onopen = function () {
			store.status = wsStatus.WAITING;
			reconnectAttempt = 0;
		};

		ws.onerror = function () {
			store.status = wsStatus.ERROR;
			console.error(`WebSocket error (${endpoint})`);
		};

		ws.onclose = function () {
			store.status = wsStatus.DISCONNECTED;
			scheduleReconnect();
		};

		ws.onmessage = async function (event) {
			store.status = wsStatus.CONNECTED;

			let responseData: string;

			if (compressed) {
				const compressedData = await event.data.arrayBuffer();
				const decompressStream = new DecompressionStream('gzip');
				const decompressedStream = new ReadableStream({
					start(controller) {
						controller.enqueue(compressedData);
						controller.close();
					}
				}).pipeThrough(decompressStream);
				responseData = await new Response(decompressedStream).text();
			} else {
				responseData = typeof event.data === 'string' ? event.data : await event.data.text();
			}

			const parsed = JSON.parse(responseData) as T;

			if (options.onMessage) {
				store.data = options.onMessage(parsed, store.data);
			} else {
				store.data = parsed;
			}
			store.lastMessageAt = Date.now();
		};
	}

	return {
		store,
		open,
		close
	};
}
