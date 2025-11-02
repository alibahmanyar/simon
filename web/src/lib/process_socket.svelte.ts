// place files you want to import through the `$lib` alias in this folder.
// import type {  } from './types';
import { ws_url } from './utils';

const ws = new WebSocket(import.meta.env.PROD ? ws_url('ws/p') : 'ws://localhost:30000/ws/p');

export const pdata = $state({
	data: {}
});

ws.onmessage = async function (event) {
	window.event = event;
	const compressedData = await event.data.arrayBuffer();
	const decompressStream = new DecompressionStream('gzip');
	const decompressedStream = new ReadableStream({
		start(controller) {
			controller.enqueue(compressedData);
			controller.close();
		}
	}).pipeThrough(decompressStream);

	const responseData = await new Response(decompressedStream).text();
	pdata.data = JSON.parse(responseData);
};
