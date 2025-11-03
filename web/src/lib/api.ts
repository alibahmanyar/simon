import { url } from './utils';
import type {
	ApiResponse,
	SystemCapabilities,
	HistoricalQueryOptions,
	HistoricalSeries,
	Alert,
	AlertVar,
	NotificationMethod
} from './types';

/**
 * Base fetch function with standardized error handling
 */
async function apiFetch<T>(
	endpoint: string,
	options?: RequestInit
): Promise<{ success: true; data: T } | { success: false; error: string }> {
	try {
		const addr = import.meta.env.PROD ? url(endpoint) : `http://localhost:30000/${endpoint}`;
		const response = await fetch(addr, options);

		if (!response.ok) {
			return {
				success: false,
				error: `HTTP error! Status: ${response.status}`
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
		const addr = import.meta.env.PROD
			? url(`container_logs/${containerId}`)
			: `http://localhost:30000/container_logs/${containerId}`;

		const response = await fetch(addr);

		if (!response.ok) {
			return {
				success: false,
				error: `Failed to fetch logs: ${response.status}`
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
