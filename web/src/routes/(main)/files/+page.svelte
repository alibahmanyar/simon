<script lang="ts">
	import { onMount } from 'svelte';
	import { fade, fly } from 'svelte/transition';
	import { getServeDirs, browseDirectory, getFileContent } from '$lib/api';
	import type { FileEntry } from '$lib/types';
	import '$lib/style-files.css';
	import {
		iconMap,
		textExtensions,
		languageMap,
		videoExtensions,
		audioExtensions,
		imageExtensions,
		documentExtensions
	} from '$lib/fileExtensions';
	import { formatBytes, url } from '$lib/utils.svelte';
	import { highlightElement, type ShjLanguage } from '@speed-highlight/core';
	import '@speed-highlight/core/themes/atom-dark.css';

	// Constants
	const MAX_TEXT_FILE_SIZE = 102400; // 100KB
	const COPY_SUCCESS_DURATION = 2000;
	const HIGHLIGHT_DELAY = 10;

	// Loading and directory state
	let is_loading = $state(true);
	let serveDirs = $state<string[]>([]);
	let currentPath = $state('');
	let currentServeDir = $state('');
	let pathSegments = $state<string[]>([]);
	let fileEntries = $state<FileEntry[]>([]);
	let error = $state<string | null>(null);

	// File viewer state
	let showFileViewer = $state(false);
	let fileContent = $state('');
	let viewingFileName = $state('');
	let viewingFilePath = $state('');
	let isLoadingFile = $state(false);
	let fileError = $state<string | null>(null);
	let copySuccess = $state(false);

	// Filter and sort state
	let filterText = $state('');
	let sortBy = $state<'name' | 'size' | 'modified'>('name');
	let sortDirection = $state<'asc' | 'desc'>('asc');

	// Derived filtered and sorted entries
	let filteredEntries = $derived.by(() => {
		// Apply filter
		let entries = filterText.trim()
			? fileEntries.filter((entry) => entry.name.toLowerCase().includes(filterText.toLowerCase()))
			: fileEntries;

		// Apply sort with directories first
		return [...entries].sort((a, b) => {
			if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;

			const comparison = sortBy === 'name' ? a.name.localeCompare(b.name) : a[sortBy] - b[sortBy];

			return sortDirection === 'asc' ? comparison : -comparison;
		});
	});

	onMount(async () => {
		const result = await getServeDirs();
		is_loading = false;

		if (result.success) {
			serveDirs = result.data;
			// Auto-browse if only one directory
			if (serveDirs.length === 1) {
				await browseDir(serveDirs[0]);
			}
		} else {
			error = result.error;
		}
	});

	async function browseDir(path: string, serveDir?: string) {
		is_loading = true;
		error = null;
		filterText = '';

		const result = await browseDirectory(path);
		is_loading = false;

		if (result.success) {
			currentPath = path;
			fileEntries = result.data.entries;
			currentServeDir = serveDir || serveDirs.find((dir) => path.startsWith(dir)) || path;

			// Calculate breadcrumb segments
			pathSegments =
				path === currentServeDir
					? []
					: path.substring(currentServeDir.length).split('/').filter(Boolean);
		} else {
			error = result.error;
		}
	}

	async function navigateToBreadcrumb(index: number) {
		const path =
			index === -1
				? currentServeDir
				: `${currentServeDir}/${pathSegments.slice(0, index + 1).join('/')}`;
		await browseDir(path);
	}

	function goBack() {
		if (currentPath === currentServeDir) {
			// Reset to directory list
			currentPath = '';
			currentServeDir = '';
			pathSegments = [];
			fileEntries = [];
		} else {
			// Navigate to parent directory
			const parentPath = currentPath.split('/').slice(0, -1).join('/');
			browseDir(parentPath);
		}
	}

	function formatDate(timestamp: number): string {
		return new Date(timestamp * 1000).toLocaleString('sv-SE', {
			year: 'numeric',
			month: '2-digit',
			day: '2-digit',
			hour: '2-digit',
			minute: '2-digit',
			second: '2-digit',
			hour12: false
		});
	}

	function getFileIcon(name: string, isDir: boolean): string {
		if (isDir) return '📁';
		const ext = name.split('.').pop()?.toLowerCase() || '';
		return iconMap[ext] || '📄';
	}

	function getFileUrl(path: string, inline = false): string {
		const params = new URLSearchParams({ path });
		if (inline) params.set('inline', 'true');
		return url(`api/files/download?${params}`);
	}

	async function downloadFile(path: string) {
		window.open(getFileUrl(path), '_blank');
	}

	function getFileExtension(filename: string): string {
		if (filename.startsWith('.'))
			// Hidden files (dot files)
			filename = filename.slice(1);
		if (!filename.includes('.')) return '';

		return filename.split('.').pop()?.toLowerCase() || '';
	}

	function isTextFile(filename: string, size: number): boolean {
		const ext = getFileExtension(filename);
		return textExtensions.includes(ext) || (!ext && size < MAX_TEXT_FILE_SIZE);
	}

	function isMediaFile(filename: string): boolean {
		const ext = getFileExtension(filename);
		return (
			videoExtensions.includes(ext) ||
			audioExtensions.includes(ext) ||
			imageExtensions.includes(ext) ||
			documentExtensions.includes(ext)
		);
	}

	async function viewFile(path: string, filename: string, size: number) {
		// Open media files in new tab
		if (isMediaFile(filename)) {
			window.open(getFileUrl(path, true), '_blank');
			return;
		}

		// Only handle text files in viewer
		if (!isTextFile(filename, size)) return;

		// Initialize file viewer
		showFileViewer = true;
		isLoadingFile = true;
		fileError = null;
		viewingFileName = filename;
		viewingFilePath = path;

		const result = await getFileContent(path);
		isLoadingFile = false;

		if (result.success) {
			fileContent = result.data;
			// Apply syntax highlighting
			setTimeout(() => {
				const codeBlock = document.querySelector('.file-viewer-content');
				if (codeBlock) {
					const language = getLanguageFromFilename(filename) as ShjLanguage;
					highlightElement(codeBlock as HTMLElement, language);
				}
			}, HIGHLIGHT_DELAY);
		} else {
			fileError = result.error;
		}
	}

	function closeFileViewer() {
		showFileViewer = false;
		fileContent = '';
		viewingFileName = '';
		viewingFilePath = '';
		fileError = null;
		copySuccess = false;
	}

	function getLanguageFromFilename(filename: string): string {
		const ext = getFileExtension(filename).toLowerCase();
		if (ext === '') return 'plain';
		return languageMap[ext] || 'plain';
	}

	function toggleSort(field: typeof sortBy) {
		sortDirection = sortBy === field && sortDirection === 'asc' ? 'desc' : 'asc';
		sortBy = field;
	}

	async function copyToClipboard() {
		try {
			await navigator.clipboard.writeText(fileContent);
			copySuccess = true;
			setTimeout(() => (copySuccess = false), COPY_SUCCESS_DURATION);
		} catch (err) {
			console.error('Failed to copy:', err);
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (showFileViewer && e.key === 'Escape') {
			closeFileViewer();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if is_loading}
	<div class="loading">
		<div class="spinner"></div>
	</div>
{:else}
	<div class="dashboard" transition:fade>
		{#if error}
			<div class="error-message" role="alert">
				<p>Error: {error}</p>
			</div>
		{/if}

		{#if serveDirs.length === 0}
			<div class="empty-state">
				<p>No directories configured for file serving</p>
				<p class="hint">Configure serve directories in the application settings</p>
			</div>
		{:else if currentPath === ''}
			<!-- Show directory list when not in any directory -->
			<div class="card">
				<p class="card-title">Available Directories</p>
				<div class="file-list">
					{#each serveDirs as dir (dir)}
						<button
							class="file-row directory-row"
							onclick={() => browseDir(dir, dir)}
							aria-label="Browse directory {dir}"
						>
							<span class="file-icon" aria-hidden="true">📁</span>
							<span class="file-name">{dir}</span>
						</button>
					{/each}
				</div>
			</div>
		{:else}
			<!-- Show file browser -->
			<div class="card">
				<div class="browser-header">
					<button class="back-button" onclick={goBack} aria-label="Go back"> ← Back </button>
					<div class="breadcrumbs" role="navigation" aria-label="Breadcrumb navigation">
						<button
							class="breadcrumb serve-dir-crumb"
							onclick={() => navigateToBreadcrumb(-1)}
							aria-label="Navigate to {currentServeDir}"
						>
							{currentServeDir}
						</button>
						{#each pathSegments as segment, i (i)}
							<span class="breadcrumb-separator" aria-hidden="true">/</span>
							<button
								class="breadcrumb"
								onclick={() => navigateToBreadcrumb(i)}
								aria-label="Navigate to {segment}"
							>
								{segment}
							</button>
						{/each}
					</div>
				</div>

				<!-- Filter and controls -->
				<div class="controls-bar">
					<input
						type="text"
						class="filter-input"
						placeholder="Filter files..."
						bind:value={filterText}
						aria-label="Filter files by name"
					/>
					<span class="results-count">
						{filteredEntries.length}
						{filteredEntries.length === 1 ? 'item' : 'items'}
					</span>
				</div>

				{#if fileEntries.length === 0}
					<div class="empty-state">
						<p>This directory is empty</p>
					</div>
				{:else if filteredEntries.length === 0}
					<div class="empty-state">
						<p>No files match your filter</p>
						<button class="clear-filter-btn" onclick={() => (filterText = '')}>Clear filter</button>
					</div>
				{:else}
					<div class="file-list">
						<div class="file-header">
							<button class="header-cell header-icon" disabled aria-label="File icon"> </button>
							<button
								class="header-cell header-sort header-name"
								onclick={() => toggleSort('name')}
								aria-label="Sort by name"
							>
								Name {sortBy === 'name' ? (sortDirection === 'asc' ? '↑' : '↓') : ''}
							</button>
							<button class="header-cell header-permissions" disabled aria-label="Permissions">
								Permissions
							</button>
							<button
								class="header-cell header-sort header-size"
								onclick={() => toggleSort('size')}
								aria-label="Sort by size"
							>
								Size {sortBy === 'size' ? (sortDirection === 'asc' ? '↑' : '↓') : ''}
							</button>
							<button
								class="header-cell header-sort header-modified"
								onclick={() => toggleSort('modified')}
								aria-label="Sort by modification date"
							>
								Modified {sortBy === 'modified' ? (sortDirection === 'asc' ? '↑' : '↓') : ''}
							</button>
							<button class="header-cell header-actions" disabled aria-label="Actions"> </button>
						</div>
						{#each filteredEntries as entry (entry.name)}
							{@const fullPath = `${currentPath}/${entry.name}`}
							{@const isText = isTextFile(entry.name, entry.size)}
							{@const isMedia = isMediaFile(entry.name)}

							<div class="file-row-wrapper">
								<button
									class="file-row"
									class:directory-row={entry.is_dir}
									class:file-item-row={!entry.is_dir}
									class:clickable-text={isText}
									class:clickable-media={isMedia}
									onclick={() =>
										entry.is_dir ? browseDir(fullPath) : viewFile(fullPath, entry.name, entry.size)}
									aria-label="{entry.is_dir
										? 'Open directory'
										: isText
											? 'View file'
											: isMedia
												? 'Open media file'
												: 'File'} {entry.name}"
								>
									<span class="file-icon" aria-hidden="true">
										{getFileIcon(entry.name, entry.is_dir)}
									</span>
									<span class="file-name">{entry.name}</span>
									<span class="file-permissions">{entry.permissions}</span>
									<span class="file-size">{entry.is_dir ? '-' : formatBytes(entry.size)}</span>
									<span class="file-modified">{formatDate(entry.modified)}</span>
									<span class="file-actions"></span>
								</button>

								{#if entry.is_dir}
									<div class="download-button-spacer"></div>
								{:else}
									<button
										class="download-button"
										onclick={() => downloadFile(fullPath)}
										title="Download {entry.name}"
										aria-label="Download {entry.name}"
									>
										↓
									</button>
								{/if}
							</div>
						{/each}
					</div>
				{/if}
			</div>
		{/if}
	</div>
{/if}

<!-- File Viewer Modal -->
{#if showFileViewer}
	<div
		class="modal-backdrop"
		onclick={closeFileViewer}
		role="presentation"
		transition:fade={{ duration: 200 }}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div
			class="modal-content"
			onclick={(e) => e.stopPropagation()}
			role="dialog"
			aria-labelledby="modal-title"
			aria-modal="true"
			tabindex="-1"
			transition:fly={{ y: 50, duration: 300 }}
		>
			<div class="modal-header">
				<div class="modal-title" id="modal-title">
					<span class="file-icon" aria-hidden="true">{getFileIcon(viewingFileName, false)}</span>
					<span>{viewingFileName}</span>
				</div>
				<div class="modal-actions">
					{#if !isLoadingFile && !fileError}
						<button
							class="modal-button"
							onclick={copyToClipboard}
							title={copySuccess ? 'Copied!' : 'Copy to clipboard'}
							aria-label="Copy file content to clipboard"
							class:success={copySuccess}
						>
							{copySuccess ? '✓' : '📋'}
						</button>
					{/if}
					<button
						class="modal-button"
						onclick={() => downloadFile(viewingFilePath)}
						title="Download file"
						aria-label="Download file"
					>
						↓
					</button>
					<button
						class="modal-button"
						onclick={closeFileViewer}
						title="Close"
						aria-label="Close file viewer"
					>
						✕
					</button>
				</div>
			</div>

			<div class="modal-body">
				{#if isLoadingFile}
					<div class="file-loading">
						<div class="spinner"></div>
						<p>Loading file...</p>
					</div>
				{:else if fileError}
					<div class="file-error">
						<p>Error: {fileError}</p>
					</div>
				{:else}
					<div class="file-viewer-content shj-lang-{getLanguageFromFilename(viewingFileName)}">
						{fileContent}
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
