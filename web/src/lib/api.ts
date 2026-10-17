import { url } from './utils.svelte';
import type {
	ApiResponse,
	SystemCapabilities,
	HistoricalQueryOptions,
	HistoricalSeries,
	Alert,
	AlertVar,
	NotificationMethod,
	DirectoryListing
} from './types';

/**
 * Pick the most useful message out of a failed response body. API endpoints
 * answer with JSON `{ error }`, a few others with plain text.
 */
function errorMessage(status: number, body: string): string {
	try {
		const parsed = JSON.parse(body);
		if (typeof parsed?.error === 'string' && parsed.error) return parsed.error;
	} catch {
		// Not JSON; fall back to the text itself
	}
	const text = body.trim();
	if (text && text.length <= 200 && !text.startsWith('<')) return text;
	return `Request failed (HTTP ${status})`;
}

async function responseError(response: Response): Promise<string> {
	const body = await response.text().catch(() => '');
	return errorMessage(response.status, body);
}

/**
 * Base fetch function with standardized error handling
 */
async function apiFetch<T>(
	endpoint: string,
	options?: RequestInit
): Promise<{ success: true; data: T } | { success: false; error: string }> {
	try {
		const response = await fetch(url(endpoint), options);

		if (!response.ok) {
			return {
				success: false,
				error: await responseError(response)
			};
		}

		const result: ApiResponse<T> = await response.json();

		if (result.success && result.data !== null) {
			return {
				success: true,
				data: result.data
			};
		} else {
			return {
				success: false,
				error: result.error || 'Unknown error occurred'
			};
		}
	} catch (error) {
		return {
			success: false,
			error: error instanceof Error ? error.message : 'Network error occurred'
		};
	}
}

/**
 * Get system capabilities
 */
export async function getSystemCapabilities() {
	return apiFetch<SystemCapabilities>('api/capabilities');
}

/**
 * Get historical data
 */
export async function getHistoricalData(options: HistoricalQueryOptions) {
	const params = new URLSearchParams();
	params.append('resolution', options.resolution);
	if (options.start_time) params.append('start_time', options.start_time.toString());
	if (options.end_time) params.append('end_time', options.end_time.toString());
	if (options.limit) params.append('limit', options.limit.toString());

	return apiFetch<HistoricalSeries[]>(`api/historical?${params.toString()}`);
}

/**
 * Get all alerts
 */
export async function getAlerts() {
	return apiFetch<Alert[]>('api/alerts');
}

/**
 * Create or update an alert
 */
export async function saveAlert(alert: Alert) {
	return apiFetch<Alert[]>('api/alerts', {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify(alert)
	});
}

/**
 * Delete an alert
 */
export async function deleteAlert(id: string) {
	return apiFetch<Alert[]>(`api/alerts/${id}`, {
		method: 'DELETE'
	});
}

/**
 * Get available alert variables
 */
export async function getAlertVars() {
	return apiFetch<AlertVar[]>('api/alert_vars');
}

/**
 * Get all notification methods
 */
export async function getNotificationMethods() {
	return apiFetch<NotificationMethod[]>('api/notif_methods');
}

/**
 * Create or update a notification method
 */
export async function saveNotificationMethod(method: NotificationMethod) {
	return apiFetch<NotificationMethod[]>('api/notif_methods', {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify(method)
	});
}

/**
 * Delete a notification method
 */
export async function deleteNotificationMethod(id: string) {
	return apiFetch<NotificationMethod[]>(`api/notif_methods/${id}`, {
		method: 'DELETE'
	});
}

/**
 * Get container logs (non-standardized endpoint, returns plain text)
 */
export async function getContainerLogs(
	containerId: string
): Promise<{ success: true; data: string[] } | { success: false; error: string }> {
	try {
		const response = await fetch(url(`container_logs/${containerId}`));

		if (!response.ok) {
			return {
				success: false,
				error: `Failed to fetch logs: ${await responseError(response)}`
			};
		}

		const text = await response.text();
		const lines = text.split('\n').filter((line) => line.trim() !== '');

		return {
			success: true,
			data: lines
		};
	} catch (error) {
		return {
			success: false,
			error: error instanceof Error ? error.message : 'Failed to load logs'
		};
	}
}

/**
 * Get configured serve directories
 */
export async function getServeDirs() {
	return apiFetch<string[]>('api/files/dirs');
}

/**
 * Browse a directory
 */
export async function browseDirectory(path: string) {
	return apiFetch<DirectoryListing>(`api/files/browse?path=${encodeURIComponent(path)}`);
}

/**
 * Get file content
 */
export async function getFileContent(
	path: string
): Promise<{ success: true; data: string } | { success: false; error: string }> {
	try {
		const response = await fetch(
			url(`api/files/download?path=${encodeURIComponent(path)}&inline=true`)
		);

		if (!response.ok) {
			return {
				success: false,
				error: `Failed to fetch file content: ${await responseError(response)}`
			};
		}

		const text = await response.text();

		return {
			success: true,
			data: text
		};
	} catch (error) {
		return {
			success: false,
			error: error instanceof Error ? error.message : 'Failed to load file content'
		};
	}
}

/**
 * Get up to `length` bytes from the end of a file, without downloading the rest.
 * `start` is the offset of the first returned byte and `size` the full file size.
 */
export async function getFileTail(
	path: string,
	length: number
): Promise<
	| { success: true; data: { bytes: Uint8Array; start: number; size: number } }
	| { success: false; error: string }
> {
	try {
		const response = await fetch(
			url(`api/files/download?path=${encodeURIComponent(path)}&inline=true`),
			{ headers: { Range: `bytes=-${length}` } }
		);

		if (!response.ok) {
			return {
				success: false,
				error: `Failed to fetch file content: ${await responseError(response)}`
			};
		}

		// e.g. "bytes 1000-1999/2000"
		const range = response.headers.get('Content-Range')?.match(/(\d+)-\d+\/(\d+)/);
		if (response.status === 206 && range) {
			const bytes = new Uint8Array(await response.arrayBuffer());
			return { success: true, data: { bytes, start: Number(range[1]), size: Number(range[2]) } };
		}

		// Range was ignored: keep only the start of the file instead of reading all of it
		const reader = response.body?.getReader();
		const chunks: Uint8Array[] = [];
		let received = 0;
		while (reader && received < length) {
			const { done, value } = await reader.read();
			if (done) break;
			chunks.push(value);
			received += value.length;
		}
		await reader?.cancel();
		const bytes = new Uint8Array(received);
		let offset = 0;
		for (const chunk of chunks) {
			bytes.set(chunk, offset);
			offset += chunk.length;
		}
		const size = Number(response.headers.get('Content-Length')) || received;
		return { success: true, data: { bytes: bytes.subarray(0, length), start: 0, size } };
	} catch (error) {
		return {
			success: false,
			error: error instanceof Error ? error.message : 'Failed to load file content'
		};
	}
}

/**
 * Upload files to a directory with progress tracking
 */
export async function uploadFiles(
	path: string,
	files: File[],
	onProgress?: (progress: number) => void
): Promise<{ success: true; data: string } | { success: false; error: string }> {
	try {
		const formData = new FormData();
		formData.append('path', path);

		// Add all files to the form data
		files.forEach((file) => {
			// Use webkitRelativePath for folder uploads, fallback to name for individual files
			const relativePath = file.webkitRelativePath || file.name;
			formData.append('file', file, relativePath);
		});

		const xhr = new XMLHttpRequest();

		return new Promise((resolve) => {
			// Track upload progress
			xhr.upload.addEventListener('progress', (e) => {
				if (e.lengthComputable && onProgress) {
					const percentComplete = (e.loaded / e.total) * 100;
					onProgress(percentComplete);
				}
			});

			xhr.addEventListener('load', () => {
				if (xhr.status === 200) {
					try {
						const result: ApiResponse<string> = JSON.parse(xhr.responseText);
						if (result.success) {
							resolve({ success: true, data: result.data || 'Upload successful' });
						} else {
							resolve({ success: false, error: result.error || 'Upload failed' });
						}
					} catch {
						resolve({ success: false, error: 'Invalid response from server' });
					}
				} else {
					resolve({
						success: false,
						error: `Upload failed: ${errorMessage(xhr.status, xhr.responseText)}`
					});
				}
			});

			xhr.addEventListener('error', () => {
				resolve({ success: false, error: 'Network error during upload' });
			});

			xhr.addEventListener('abort', () => {
				resolve({ success: false, error: 'Upload cancelled' });
			});

			xhr.open('POST', url('api/files/upload'));
			xhr.send(formData);
		});
	} catch (error) {
		return {
			success: false,
			error: error instanceof Error ? error.message : 'Failed to upload files'
		};
	}
}

/**
 * Move/rename a file or folder
 */
export async function moveFile(source: string, destination: string) {
	return apiFetch<string>(`api/files/move`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ source, destination })
	});
}

/**
 * Delete a file or folder
 */
export async function deleteFile(path: string) {
	return apiFetch<string>(`api/files/delete`, {
		method: 'DELETE',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ path })
	});
}

/**
 * Create a new folder
 */
export async function createFolder(path: string, name: string) {
	return apiFetch<string>(`api/files/create_folder`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ path, name })
	});
}
