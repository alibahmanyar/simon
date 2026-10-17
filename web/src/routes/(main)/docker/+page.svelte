<script lang="ts">
	import '$lib/style-docker.css';
	import { formatBytes, capabilities } from '$lib/utils.svelte';
	import { ddata } from '$lib/docker_socket.svelte';
	import Modal from '$lib/Modal.svelte';

	import { wsStatus } from '$lib/types';
	import { getContainerLogs } from '$lib/api';
	// $inspect(ddata).with(console.trace);

	// Create state for container filtering and search
	let searchTerm = $state('');
	let activeFilter = $state('all');

	// State for container logs modal
	let showLogsModal = $state(false);
	let selectedContainer = $state<{ id: string; name: string } | null>(null);
	let containerLogs = $state<string[]>([]);
	let isLoadingLogs = $state(false);

	// Reference to logs container element
	let logsContainer = $state<HTMLDivElement | null>(null);

	// Function to scroll logs to bottom
	function scrollLogsToBottom() {
		if (logsContainer) {
			logsContainer.scrollTop = logsContainer.scrollHeight;
		}
	}

	// Use $effect (runes compatible) to scroll logs when they're loaded
	$effect(() => {
		if (showLogsModal && !isLoadingLogs && containerLogs.length > 0 && logsContainer) {
			scrollLogsToBottom();
		}
	});

	// Function to format timestamp to readable date
	function formatTimestamp(timestamp: number) {
		const date = new Date(timestamp * 1000);
		return date.toLocaleString();
	}

	// Function to handle filter button click
	function setFilter(filter: string) {
		activeFilter = filter;
	}

	// Function to fetch container logs
	async function fetchContainerLogs(containerId: string) {
		isLoadingLogs = true;
		const result = await getContainerLogs(containerId);
		isLoadingLogs = false;

		if (result.success) {
			return result.data;
		} else {
			console.error('Error fetching container logs:', result.error);
			return ['Failed to load logs. Please try again.'];
		}
	}

	// Function to reload logs for the current container
	async function reloadLogs() {
		if (selectedContainer) {
			containerLogs = await fetchContainerLogs(selectedContainer.id);
		}
	}

	// Function to format log line
	function formatLogLine(line: string) {
		if (!line || line.length < 10) return { type: 'unknown', text: line };

		const parts = line.split('|');
		if (parts.length < 2) return { type: 'unknown', text: line };

		const type = parts[0]; // STDOUT or STDERR
		const timestamp = parts[1].substring(0, 19); // Remove milliseconds
		const message = parts
			.slice(1)
			.join('|')
			.substring(parts[1].indexOf('Z') + 2);

		return {
			type,
			timestamp,
			text: message
		};
	}

	// Function to open logs modal
	async function openLogsModal(container: { id: string; name: string }) {
		selectedContainer = container;
		showLogsModal = true;
		containerLogs = await fetchContainerLogs(container.id);
	}

	// Function to close logs modal
	function closeLogsModal() {
		showLogsModal = false;
		selectedContainer = null;
		containerLogs = [];
	}

	// Computed property for filtered containers
	let filteredContainers = $derived(
		ddata.data?.containers
			?.filter((container) => {
				// Check search term
				const matchesSearch =
					container.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
					container.image.toLowerCase().includes(searchTerm.toLowerCase()) ||
					container.id.toLowerCase().includes(searchTerm.toLowerCase());

				// Check status filter
				const matchesFilter =
					activeFilter === 'all' ||
					(activeFilter === 'running' && container.state === 'running') ||
					(activeFilter === 'exited' && container.state === 'exited') ||
					(activeFilter === 'created' && container.state === 'created');

				return matchesSearch && matchesFilter;
			})
			.slice()
			.sort((a, b) => {
				// Sort by status - running containers first then created ones
				if (a.state === 'running' && b.state !== 'running') return -1;
				if (a.state !== 'running' && b.state === 'running') return 1;
				if (a.state === 'created' && b.state !== 'created') return -1;
				if (a.state !== 'created' && b.state === 'created') return 1;
				// If same status, sort alphabetically by name
				return a.name.localeCompare(b.name);
			}) || []
	);
</script>

<div class="dashboard">
	{#if !capabilities.docker || ddata.data === null}
		<div class="error-container">
			<p>Docker Not Available</p>
			<p>
				Simon can't reach the Docker API. Make sure Docker is running and the user running Simon can
				access <code>/var/run/docker.sock</code> (for example by being in the docker group). When
				Simon runs in a container, mount the socket with
				<code>-v /var/run/docker.sock:/var/run/docker.sock:ro</code>.
			</p>
		</div>
	{:else if (ddata.data?.containers?.length ?? 0) > 0}
		<!-- Containers stay on screen through a reconnect, marked as possibly stale -->
		<div class="card">
			<p class="card-title">Docker Containers</p>
			{#if ddata.status !== wsStatus.CONNECTED}
				<p class="stale-note" role="status">Reconnecting… container data may be out of date.</p>
			{/if}

			<div class="container-filters">
				<input
					type="text"
					class="container-search"
					placeholder="Search containers..."
					aria-label="Search containers"
					bind:value={searchTerm}
				/>
				<button
					class="container-filter-btn {activeFilter === 'all' ? 'active' : ''}"
					aria-pressed={activeFilter === 'all'}
					onclick={() => setFilter('all')}>All</button
				>
				<button
					class="container-filter-btn {activeFilter === 'running' ? 'active' : ''}"
					aria-pressed={activeFilter === 'running'}
					onclick={() => setFilter('running')}>Running</button
				>
				<button
					class="container-filter-btn {activeFilter === 'exited' ? 'active' : ''}"
					aria-pressed={activeFilter === 'exited'}
					onclick={() => setFilter('exited')}>Exited</button
				>
				<button
					class="container-filter-btn {activeFilter === 'created' ? 'active' : ''}"
					aria-pressed={activeFilter === 'created'}
					onclick={() => setFilter('created')}>Created</button
				>
			</div>

			{#if filteredContainers.length > 0}
				<div class="container-grid">
					{#each filteredContainers as container (container.id)}
						{@const name = container.name.replace(/^\//, '')}
						{@const memoryPercentage =
							container.mem_limit > 0
								? ((container.mem_usage / container.mem_limit) * 100).toFixed(1)
								: 'N/A'}
						{@const statusClass = container.state.toLowerCase()}

						<div class="container-item">
							<div class="container-header">
								<span class="container-name">{name}</span>
								<span class="container-status {statusClass}">{container.state}</span>
							</div>
							<div class="container-image">{container.image}</div>
							<div class="container-detail">
								<div class="container-detail-label">Status</div>
								{container.status}
							</div>
							<div class="container-detail">
								<div class="container-detail-label">Created</div>
								{formatTimestamp(container.created)}
							</div>

							{#if container.state === 'running'}
								<div class="container-detail">
									<div class="container-detail-label">
										CPU Usage: {container.cpu_usage.toFixed(2)}%
									</div>
									<div class="container-usage">
										<div
											class="container-cpu-fill"
											style="width: {Math.min(100, container.cpu_usage)}%"
										></div>
									</div>
								</div>

								<div class="container-detail">
									<div class="container-detail-label">
										Memory Usage: {formatBytes(container.mem_usage)}
										{#if container.mem_limit > 0}
											/ {formatBytes(container.mem_limit)} ({memoryPercentage}%)
										{/if}
									</div>
									<div class="container-usage">
										<div
											class="container-memory-fill"
											style="width: {container.mem_limit > 0
												? Math.min(100, (container.mem_usage / container.mem_limit) * 100)
												: 0}%"
										></div>
									</div>
								</div>
							{/if}

							{#if container.ports && container.ports.length > 0}
								<div class="container-detail">
									<div class="container-detail-label">Ports</div>
									<div>
										{#each container.ports.toSorted((a, b) => (a.pub_port || 0) - (b.pub_port || 0) || a.priv_port - b.priv_port) as port, i (i)}
											<span class="container-port">
												{port.ip ? port.ip + ':' : ''}{port.pub_port ||
													''}:{port.priv_port}/{port.protocol}
											</span>
										{/each}
									</div>
								</div>
							{/if}

							<div class="container-detail">
								<div class="container-detail-label">ID</div>
								<div style="font-size: 0.8rem; opacity: 0.7; overflow:hidden; max-width: 80%;">
									{container.id}
								</div>
							</div>
							<button
								class="container-logs-button"
								onclick={() => openLogsModal({ id: container.id, name })}>View Logs</button
							>
						</div>
					{/each}
				</div>
			{:else}
				<div class="info-item">
					{#if !ddata.data?.containers?.length}
						<p>No Docker containers found</p>
					{:else}
						<p>No containers match your filters</p>
					{/if}
				</div>
			{/if}
		</div>
	{:else if ddata.data?.containers}
		<div class="info-item">
			<p>No Docker containers found on system</p>
		</div>
	{:else if ddata.status === wsStatus.DISCONNECTED || ddata.status === wsStatus.ERROR}
		<div class="error-container">
			<p>Connection failed</p>
			<p>Could not reach the Simon server. Retrying automatically…</p>
		</div>
	{:else}
		<div class="loading">
			<div class="spinner"></div>
			<p>Waiting for container info...</p>
		</div>
	{/if}
</div>

<!-- Logs Modal -->
{#if showLogsModal && selectedContainer}
	<Modal class="logs-modal-overlay" aria-labelledby="logs-modal-title" onclose={closeLogsModal}>
		<div class="logs-modal">
			<div class="logs-modal-header">
				<div class="logs-modal-title-section">
					<h3 id="logs-modal-title">Logs: {selectedContainer.name}</h3>
					<button
						class="logs-reload-button"
						onclick={reloadLogs}
						title="Reload logs"
						aria-label="Reload logs">↻</button
					>
				</div>
				<button class="logs-close-button" onclick={closeLogsModal} aria-label="Close logs">×</button
				>
			</div>
			<div class="logs-modal-content" bind:this={logsContainer}>
				{#if isLoadingLogs}
					<div class="logs-loading">Loading logs...</div>
				{:else if containerLogs.length === 0}
					<div class="logs-empty">No logs available</div>
				{:else}
					<div class="logs-container">
						{#each containerLogs as logLine, i (i)}
							{@const log = formatLogLine(logLine)}
							<div class="log-line {log.type.toLowerCase()}">
								{#if log.timestamp}
									<span class="log-timestamp">[{log.timestamp}]</span>
								{/if}
								<span class="log-text">{log.text}</span>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	</Modal>
{/if}
