import { ws_url } from './utils';
import { wsStatus, type SystemData } from './types';

export const gdata = $state({
    data: null as SystemData | null,
    prevDataPoints: [] as SystemData[],
    status: wsStatus.INIT as wsStatus
});

let g_ws: WebSocket | null = null;
let reconnectAttempt = 0;
let reconnectTimeout: number | null = null;
let isReconnecting = false;

function getReconnectDelay(): number {
    // Backoff with a maximum of 10 seconds
    return Math.min(2000 * (reconnectAttempt + 1), 10000);
}

function scheduleReconnect() {
    if (isReconnecting) return;
    
    isReconnecting = true;
    const delay = getReconnectDelay();
    
    //console.log(`Scheduling reconnect attempt in ${delay}ms`);
    
    if (reconnectTimeout !== null) {
        clearTimeout(reconnectTimeout);
    }
    
    reconnectTimeout = setTimeout(() => {
        reconnectAttempt++;;
        isReconnecting = false;
        open_ws();
    }, delay) as unknown as number;
}

export function close_ws() {
    if (g_ws !== null) {
        g_ws.close();
    }
}

export function open_ws() {
    // Clean up existing connection if any
    if (g_ws !== null) {
        try {
            g_ws.onopen = null;
            g_ws.onclose = null;
            g_ws.onerror = null;
            g_ws.onmessage = null;
            g_ws.close();
        } catch (e) {
            console.error("Error while closing existing WebSocket:", e);
        }
        g_ws = null;
    }

    try {
        g_ws = new WebSocket(import.meta.env.PROD
            ? ws_url('ws/g')
            : "ws://localhost:30000/ws/g");
    }
    catch (e) {
        console.error("WebSocket connection failed: ", e);
        gdata.status = wsStatus.ERROR;
        scheduleReconnect();
        return;
    }

    g_ws.onopen = function (event) {
        gdata.status = wsStatus.WAITING;
        //console.log("WebSocket opened:", event);
        reconnectAttempt = 0;
    }

    g_ws.onerror = function (event) {
        gdata.status = wsStatus.ERROR;
        console.error("WebSocket error observed:", event);
        // We'll let onclose handle the reconnection
    }

    g_ws.onclose = function (event) {
        gdata.status = wsStatus.DISCONNECTED;
        //console.log("WebSocket closed:", event);
        scheduleReconnect();
    }

    g_ws.onmessage = async function (event) {
        gdata.status = wsStatus.CONNECTED;
        const compressedData = await event.data.arrayBuffer();
        const decompressStream = new DecompressionStream('gzip');
        const decompressedStream = new ReadableStream({
            start(controller) {
                controller.enqueue(compressedData);
                controller.close();
            }
        }).pipeThrough(decompressStream);

        const responseData = await new Response(decompressedStream).text();

        if (gdata.data !== null)
            gdata.prevDataPoints.push(gdata.data);

        if (gdata.prevDataPoints.length > 60) {
            gdata.prevDataPoints.shift();
        }
        gdata.data = JSON.parse(responseData) as SystemData;
    }
}