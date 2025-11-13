<script lang="ts">
	import { onMount } from 'svelte';
	import { fade, fly } from 'svelte/transition';
	import {
		getServeDirs,
		browseDirectory,
		getFileContent,
		uploadFiles,
		moveFile,
		deleteFile,
		createFolder
	} from '$lib/api';
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

	// TypeScript definitions for FileSystem API
	interface FileSystemEntry {
		isFile: boolean;
		isDirectory: boolean;
		name: string;
		fullPath: string;
	}

	interface FileSystemFileEntry extends FileSystemEntry {
		file(successCallback: (file: File) => void, errorCallback?: (error: Error) => void): void;
	}

	interface FileSystemDirectoryEntry extends FileSystemEntry {
		createReader(): FileSystemDirectoryReader;
	}

	interface FileSystemDirectoryReader {
		readEntries(
			successCallback: (entries: FileSystemEntry[]) => void,
			errorCallback?: (error: Error) => void
		): void;
	}

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

	// Upload state
	let showUploadModal = $state(false);
	let isUploading = $state(false);
	let uploadProgress = $state(0);
	let uploadError = $state<string | null>(null);
	let uploadSuccess = $state<string | null>(null);
	let fileInputRef: HTMLInputElement;
	let folderInputRef: HTMLInputElement;
	let isDragging = $state(false);
	let dragCounter = $state(0);

	// Filter and sort state
	let filterText = $state('');
	let sortBy = $state<'name' | 'size' | 'modified'>('name');
	let sortDirection = $state<'asc' | 'desc'>('asc');

	// Options menu state
	let showOptionsMenu = $state<string | null>(null); // Stores the path of the file with open menu

	// Rename modal state
	let showRenameModal = $state(false);
	let renameFilePath = $state('');
	let renameFileName = $state('');
	let newFileName = $state('');
	let isRenaming = $state(false);
	let renameError = $state<string | null>(null);

	// Move modal state
	let showMoveModal = $state(false);
	let moveFilePath = $state('');
	let moveFileName = $state('');
	let moveDestination = $state('');
	let isMoving = $state(false);
	let moveError = $state<string | null>(null);

	// Create folder modal state
	let showCreateFolderModal = $state(false);
	let newFolderName = $state('');
	let isCreatingFolder = $state(false);
	let createFolderError = $state<string | null>(null);

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
		let ext = getFileExtension(filename).toLowerCase();
		if (ext === '') ext = filename.toLowerCase();
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
		if (showUploadModal && e.key === 'Escape' && !isUploading) {
			closeUploadModal();
		}
		if (showRenameModal && e.key === 'Escape' && !isRenaming) {
			closeRenameModal();
		}
		if (showMoveModal && e.key === 'Escape' && !isMoving) {
			closeMoveModal();
		}
		if (showCreateFolderModal && e.key === 'Escape' && !isCreatingFolder) {
			closeCreateFolderModal();
		}
		if (showOptionsMenu && e.key === 'Escape') {
			showOptionsMenu = null;
		}
	}

	function toggleOptionsMenu(path: string, e: MouseEvent) {
		e.stopPropagation();
		showOptionsMenu = showOptionsMenu === path ? null : path;
	}

	function handleRename(path: string, currentName: string) {
		showOptionsMenu = null;
		showRenameModal = true;
		renameFilePath = path;
		renameFileName = currentName;
		newFileName = currentName;
		renameError = null;
	}

	function handleMove(path: string, name: string) {
		showOptionsMenu = null;
		showMoveModal = true;
		moveFilePath = path;
		moveFileName = name;
		moveDestination = currentPath; // Default to current directory
		moveError = null;
	}

	async function handleDelete(path: string, name: string, isDir: boolean) {
		showOptionsMenu = null;
		const itemType = isDir ? 'folder' : 'file';
		const confirmed = confirm(
			`Are you sure you want to delete this ${itemType} "${name}"?\n\nThis action cannot be undone.`
		);
		if (!confirmed) return;

		const result = await deleteFile(path);

		if (result.success) {
			// Refresh the directory listing
			await browseDir(currentPath);
		} else {
			alert(`Failed to delete ${itemType}: ${result.error}`);
		}
	}

	async function performRename() {
		if (!newFileName.trim() || newFileName === renameFileName) {
			renameError = 'Please enter a different name';
			return;
		}

		isRenaming = true;
		renameError = null;

		// Construct the destination path (same directory, new name)
		const pathParts = renameFilePath.split('/');
		pathParts[pathParts.length - 1] = newFileName.trim();
		const destination = pathParts.join('/');

		const result = await moveFile(renameFilePath, destination);
		isRenaming = false;

		if (result.success) {
			showRenameModal = false;
			// Refresh the directory listing
			await browseDir(currentPath);
		} else {
			renameError = result.error;
		}
	}

	async function performMove() {
		if (!moveDestination.trim()) {
			moveError = 'Please enter a destination path';
			return;
		}

		isMoving = true;
		moveError = null;

		const result = await moveFile(moveFilePath, moveDestination.trim());
		isMoving = false;

		if (result.success) {
			showMoveModal = false;
			// Refresh the directory listing
			await browseDir(currentPath);
		} else {
			moveError = result.error;
		}
	}

	function closeRenameModal() {
		if (isRenaming) return;
		showRenameModal = false;
		renameError = null;
	}

	function closeMoveModal() {
		if (isMoving) return;
		showMoveModal = false;
		moveError = null;
	}

	function openCreateFolderModal() {
		showCreateFolderModal = true;
		newFolderName = '';
		createFolderError = null;
	}

	function closeCreateFolderModal() {
		if (isCreatingFolder) return;
		showCreateFolderModal = false;
		newFolderName = '';
		createFolderError = null;
	}

	async function performCreateFolder() {
		if (!newFolderName.trim()) {
			createFolderError = 'Please enter a folder name';
			return;
		}

		isCreatingFolder = true;
		createFolderError = null;

		const result = await createFolder(currentPath, newFolderName.trim());
		isCreatingFolder = false;

		if (result.success) {
			showCreateFolderModal = false;
			// Refresh the directory listing
			await browseDir(currentPath);
		} else {
			createFolderError = result.error;
		}
	}

	function openUploadModal() {
		showUploadModal = true;
		uploadError = null;
		uploadSuccess = null;
		uploadProgress = 0;
	}

	function closeUploadModal() {
		if (isUploading) return; // Prevent closing during upload
		showUploadModal = false;
		uploadError = null;
		uploadSuccess = null;
		uploadProgress = 0;
	}

	async function handleFileUpload(files: FileList | null) {
		if (!files || files.length === 0) return;

		isUploading = true;
		uploadError = null;
		uploadSuccess = null;
		uploadProgress = 0;

		const fileArray = Array.from(files);
		const result = await uploadFiles(currentPath, fileArray, (progress) => {
			uploadProgress = progress;
		});

		isUploading = false;

		if (result.success) {
			uploadSuccess = result.data;
			uploadProgress = 100;
			// Refresh the directory listing
			setTimeout(async () => {
				await browseDir(currentPath);
				closeUploadModal();
			}, 1500);
		} else {
			uploadError = result.error;
		}
	}

	function triggerFileUpload() {
		fileInputRef?.click();
	}

	function triggerFolderUpload() {
		folderInputRef?.click();
	}

	function handleDragEnter(e: DragEvent) {
		e.preventDefault();
		e.stopPropagation();
		dragCounter++;
		if (e.dataTransfer?.types.includes('Files')) {
			isDragging = true;
		}
	}

	function handleDragLeave(e: DragEvent) {
		e.preventDefault();
		e.stopPropagation();
		dragCounter--;
		if (dragCounter === 0) {
			isDragging = false;
		}
	}

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
		e.stopPropagation();
		if (e.dataTransfer) {
			e.dataTransfer.dropEffect = 'copy';
		}
	}

	async function handleDrop(e: DragEvent) {
		e.preventDefault();
		e.stopPropagation();
		isDragging = false;
		dragCounter = 0;

		const items = e.dataTransfer?.items;
		if (!items) return;

		const files: File[] = [];

		// Process all dropped items (files and folders)
		for (let i = 0; i < items.length; i++) {
			const item = items[i];
			if (item.kind === 'file') {
				const entry = item.webkitGetAsEntry();
				if (entry) {
					await processEntry(entry, files);
				}
			}
		}

		if (files.length > 0) {
			// Create a FileList-like object
			const dataTransfer = new DataTransfer();
			files.forEach((file) => dataTransfer.items.add(file));
			handleFileUpload(dataTransfer.files);
		}
	}

	async function processEntry(entry: FileSystemEntry, files: File[], path = '') {
		if (entry.isFile) {
			const fileEntry = entry as FileSystemFileEntry;
			const file = await new Promise<File>((resolve, reject) => {
				fileEntry.file(resolve, reject);
			});
			// Create a new File object with the full path
			const fullPath = path ? `${path}/${file.name}` : file.name;
			const newFile = new File([file], fullPath, { type: file.type });
			files.push(newFile);
		} else if (entry.isDirectory) {
			const dirEntry = entry as FileSystemDirectoryEntry;
			const reader = dirEntry.createReader();
			const entries = await new Promise<FileSystemEntry[]>((resolve, reject) => {
				reader.readEntries(resolve, reject);
			});
			const newPath = path ? `${path}/${entry.name}` : entry.name;
			for (const childEntry of entries) {
				await processEntry(childEntry, files, newPath);
			}
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} onclick={() => (showOptionsMenu = null)} />

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
					<div class="header-actions">
						<button
							class="action-button"
							onclick={openCreateFolderModal}
							aria-label="Create new folder"
							title="Create folder"
						>
							📁 New Folder
						</button>
						<button
							class="action-button upload-button"
							onclick={openUploadModal}
							aria-label="Upload files"
							title="Upload files"
						>
							📤 Upload
						</button>
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

								<div class="action-buttons-container">
									<button
										class="more-button"
										onclick={(e) => toggleOptionsMenu(fullPath, e)}
										title="More options"
										aria-label="More options for {entry.name}"
										aria-expanded={showOptionsMenu === fullPath}
									>
										⋮
									</button>

									{#if showOptionsMenu === fullPath}
										<div class="options-menu" transition:fade={{ duration: 150 }}>
											<button
												class="option-item"
												onclick={(e) => {
													e.stopPropagation();
													handleRename(fullPath, entry.name);
												}}
											>
												<span class="option-icon">✏️</span>
												<span>Rename</span>
											</button>
											<button
												class="option-item"
												onclick={(e) => {
													e.stopPropagation();
													handleMove(fullPath, entry.name);
												}}
											>
												<span class="option-icon">📁</span>
												<span>Move</span>
											</button>
											<button
												class="option-item option-item-danger"
												onclick={(e) => {
													e.stopPropagation();
													handleDelete(fullPath, entry.name, entry.is_dir);
												}}
											>
												<span class="option-icon">🗑️</span>
												<span>Delete</span>
											</button>
										</div>
									{/if}
								</div>
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
			class="modal-content file-viewer-modal"
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

<!-- Upload Modal -->
{#if showUploadModal}
	<div
		class="modal-backdrop"
		onclick={() => !isUploading && closeUploadModal()}
		role="presentation"
		transition:fade={{ duration: 200 }}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div
			class="modal-content upload-modal"
			onclick={(e) => e.stopPropagation()}
			role="dialog"
			aria-labelledby="upload-modal-title"
			aria-modal="true"
			tabindex="-1"
			transition:fly={{ y: 50, duration: 300 }}
		>
			<div class="modal-header">
				<div class="modal-title" id="upload-modal-title">
					<span aria-hidden="true">📤</span>
					<span>Upload Files or Folders</span>
				</div>
				<button
					class="modal-button"
					onclick={closeUploadModal}
					disabled={isUploading}
					title="Close"
					aria-label="Close upload dialog"
				>
					✕
				</button>
			</div>

			<div class="modal-body">
				{#if uploadSuccess}
					<div class="upload-success">
						<p>✓ {uploadSuccess}</p>
					</div>
				{:else if uploadError}
					<div class="upload-error">
						<p>✗ {uploadError}</p>
					</div>
				{/if}

				<div class="upload-destination">
					<strong>Upload to:</strong>
					<code>{currentPath}</code>
				</div>

				{#if isUploading}
					<div class="upload-progress-container">
						<div class="upload-progress-bar">
							<div class="upload-progress-fill" style="width: {uploadProgress}%"></div>
						</div>
						<div class="upload-progress-text">{Math.round(uploadProgress)}%</div>
					</div>
				{:else}
					<!-- Drag and Drop Zone -->
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div
						class="drag-drop-zone"
						class:dragging={isDragging}
						ondragenter={handleDragEnter}
						ondragleave={handleDragLeave}
						ondragover={handleDragOver}
						ondrop={handleDrop}
					>
						<div class="drag-drop-content">
							<span class="drag-drop-icon">📄</span>
							<p class="drag-drop-title">Drag and drop files or folders here</p>
							<p class="drag-drop-subtitle">or</p>
						</div>
					</div>

					<div class="upload-actions">
						<button class="upload-action-button" onclick={triggerFileUpload}>
							<span class="upload-icon">📄</span>
							<span>Select Files</span>
						</button>
						<button class="upload-action-button" onclick={triggerFolderUpload}>
							<span class="upload-icon">📁</span>
							<span>Select Folder</span>
						</button>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

<!-- Rename Modal -->
{#if showRenameModal}
	<div
		class="modal-backdrop"
		onclick={closeRenameModal}
		role="presentation"
		transition:fade={{ duration: 200 }}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div
			class="modal-content rename-modal"
			onclick={(e) => e.stopPropagation()}
			role="dialog"
			aria-labelledby="rename-modal-title"
			aria-modal="true"
			tabindex="-1"
			transition:fly={{ y: 50, duration: 300 }}
		>
			<div class="modal-header">
				<div class="modal-title" id="rename-modal-title">
					<span aria-hidden="true">✏️</span>
					<span>Rename</span>
				</div>
				<button
					class="modal-button"
					onclick={closeRenameModal}
					disabled={isRenaming}
					title="Close"
					aria-label="Close rename dialog"
				>
					✕
				</button>
			</div>

			<div class="modal-body">
				{#if renameError}
					<div class="upload-error">
						<p>✗ {renameError}</p>
					</div>
				{/if}

				<div class="form-group">
					<label for="rename-input">New name:</label>
					<input
						id="rename-input"
						type="text"
						class="text-input"
						bind:value={newFileName}
						disabled={isRenaming}
						onkeydown={(e) => e.key === 'Enter' && performRename()}
						aria-label="New file or folder name"
					/>
				</div>

				<div class="modal-footer">
					<button class="secondary-button" onclick={closeRenameModal} disabled={isRenaming}>
						Cancel
					</button>
					<button class="primary-button" onclick={performRename} disabled={isRenaming}>
						{isRenaming ? 'Renaming...' : 'Rename'}
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}

<!-- Move Modal -->
{#if showMoveModal}
	<div
		class="modal-backdrop"
		onclick={closeMoveModal}
		role="presentation"
		transition:fade={{ duration: 200 }}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div
			class="modal-content move-modal"
			onclick={(e) => e.stopPropagation()}
			role="dialog"
			aria-labelledby="move-modal-title"
			aria-modal="true"
			tabindex="-1"
			transition:fly={{ y: 50, duration: 300 }}
		>
			<div class="modal-header">
				<div class="modal-title" id="move-modal-title">
					<span aria-hidden="true">📁</span>
					<span>Move</span>
				</div>
				<button
					class="modal-button"
					onclick={closeMoveModal}
					disabled={isMoving}
					title="Close"
					aria-label="Close move dialog"
				>
					✕
				</button>
			</div>

			<div class="modal-body">
				{#if moveError}
					<div class="upload-error">
						<p>✗ {moveError}</p>
					</div>
				{/if}

				<div class="form-group">
					<label for="move-source">Moving:</label>
					<div class="read-only-field">
						<code>{moveFileName}</code>
					</div>
				</div>

				<div class="form-group">
					<label for="move-destination">Destination path:</label>
					<input
						id="move-destination"
						type="text"
						class="text-input"
						bind:value={moveDestination}
						disabled={isMoving}
						onkeydown={(e) => e.key === 'Enter' && performMove()}
						placeholder="Enter full destination path"
						aria-label="Destination path"
					/>
				</div>

				<div class="modal-footer">
					<button class="secondary-button" onclick={closeMoveModal} disabled={isMoving}>
						Cancel
					</button>
					<button class="primary-button" onclick={performMove} disabled={isMoving}>
						{isMoving ? 'Moving...' : 'Move'}
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}

<!-- Create Folder Modal -->
{#if showCreateFolderModal}
	<div
		class="modal-backdrop"
		onclick={closeCreateFolderModal}
		role="presentation"
		transition:fade={{ duration: 200 }}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div
			class="modal-content create-folder-modal"
			onclick={(e) => e.stopPropagation()}
			role="dialog"
			aria-labelledby="create-folder-modal-title"
			aria-modal="true"
			tabindex="-1"
			transition:fly={{ y: 50, duration: 300 }}
		>
			<div class="modal-header">
				<div class="modal-title" id="create-folder-modal-title">
					<span aria-hidden="true">📁</span>
					<span>Create New Folder</span>
				</div>
				<button
					class="modal-button"
					onclick={closeCreateFolderModal}
					disabled={isCreatingFolder}
					title="Close"
					aria-label="Close create folder dialog"
				>
					✕
				</button>
			</div>

			<div class="modal-body">
				{#if createFolderError}
					<div class="upload-error">
						<p>✗ {createFolderError}</p>
					</div>
				{/if}

				<div class="form-group">
					<label for="folder-name-input">Folder name:</label>
					<input
						id="folder-name-input"
						type="text"
						class="text-input"
						bind:value={newFolderName}
						disabled={isCreatingFolder}
						onkeydown={(e) => e.key === 'Enter' && performCreateFolder()}
						placeholder="Enter folder name"
						aria-label="New folder name"
					/>
				</div>

				<div class="upload-destination">
					<strong>Create in:</strong>
					<code>{currentPath}</code>
				</div>

				<div class="modal-footer">
					<button
						class="secondary-button"
						onclick={closeCreateFolderModal}
						disabled={isCreatingFolder}
					>
						Cancel
					</button>
					<button class="primary-button" onclick={performCreateFolder} disabled={isCreatingFolder}>
						{isCreatingFolder ? 'Creating...' : 'Create Folder'}
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}

<!-- Hidden file inputs -->
<input
	type="file"
	bind:this={fileInputRef}
	onchange={(e) => handleFileUpload(e.currentTarget.files)}
	multiple
	style="display: none"
	aria-hidden="true"
/>
<input
	type="file"
	bind:this={folderInputRef}
	onchange={(e) => handleFileUpload(e.currentTarget.files)}
	webkitdirectory
	multiple
	style="display: none"
	aria-hidden="true"
/>
