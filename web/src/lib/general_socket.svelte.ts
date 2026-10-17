import { createWebSocketStore } from './ws_store.svelte';
import type { SystemData } from './types';

/** Track previous data points for chart history */
const MAX_HISTORY = 60;

interface GeneralStoreData {
	current: SystemData | null;
	prevDataPoints: SystemData[];
}

const generalSocket = createWebSocketStore<GeneralStoreData>('ws/g', {
	initialData: { current: null, prevDataPoints: [] },
	onMessage: (parsed: unknown, previous: GeneralStoreData | null) => {
		const systemData = parsed as SystemData;
		systemData.receivedAt = Date.now();
		const prev = previous ?? { current: null, prevDataPoints: [] };
		const prevDataPoints = [...prev.prevDataPoints];

		if (prev.current !== null) {
			prevDataPoints.push(prev.current);
		}

		if (prevDataPoints.length > MAX_HISTORY) {
			prevDataPoints.shift();
		}

		return {
			current: systemData,
			prevDataPoints
		};
	}
});

/**
 * Reactive store for general system data.
 * Access: gdata.data.current, gdata.data.prevDataPoints, gdata.status
 */
export const gdata = generalSocket.store;

/** Open the general WebSocket connection */
export const open_ws = generalSocket.open;

/** Close the general WebSocket connection */
export const close_ws = generalSocket.close;
