<script lang="ts">
	import '$lib/style-settings.css';
	import type { NotificationMethod } from '$lib/types';
	import {
		getNotificationMethods,
		saveNotificationMethod,
		deleteNotificationMethod as deleteNotificationMethodApi
	} from '$lib/api';
	import { onMount } from 'svelte';
	import { fly, fade } from 'svelte/transition';

	let is_loading: boolean = $state(true);
	let notificationMethods: NotificationMethod[] = $state([]);
	let showDialog = $state(false);

	let is_new = $state(true);

	// Form data
	let webhookForm: NotificationMethod = $state({
		id: '-1',
		name: '',
		kind: 'webhook',
		config: {
			WebHook: {
				url: '',
				method: 'POST',
				headers: {},
				body: ''
			}
		},
		enabled: true
	});

	let headersString = $state('');

	let showBodyTextArea = $state(true);

	// Methods
	const methods = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE'];

	onMount(async () => {
		is_loading = true;
		const result = await getNotificationMethods();
		is_loading = false;

		if (result.success) {
			notificationMethods = result.data;
		} else {
			console.error('Failed to fetch notification methods:', result.error);
		}
	});

	function toggleDialog() {
		showDialog = !showDialog;
		if (showDialog) {
			if (is_new) {
				// Reset form when opening
				webhookForm = {
					id: '-1',
					name: '',
					kind: 'webhook',
					config: {
						WebHook: {
							url: '',
							method: 'POST',
							headers: {},
							body: ''
						}
					},
					enabled: true
				};
			}
		}
	}

	async function addNotificationMethod() {
		webhookForm.config.WebHook.headers = headersString
			.split('\n')
			.reduce((res: Record<string, string>, line) => {
				let [key, value] = line.split(':').map((part) => part.trim());
				if (key && value) res[key] = value;
				return res;
			}, {});

		// Create a new notification source with WebHook config
		const newMethod = webhookForm;

		is_loading = true;
		const result = await saveNotificationMethod(newMethod);
		is_loading = false;

		if (result.success) {
			notificationMethods = result.data;
			toggleDialog();
		} else {
			console.error('Error adding notification method:', result.error);
			alert(`Failed to add notification method: ${result.error}`);
		}
	}

	let testResultMessage = $state('');
	let testResultStatus: 'idle' | 'loading' | 'success' | 'error' = $state('idle');

	function sendTestNotification() {
		if (webhookForm.config.WebHook.url.length === 0) {
			testResultMessage = 'Webhook URL is required';
			testResultStatus = 'error';
			return;
		}

		testResultMessage = 'Sending test notification...';
		testResultStatus = 'loading';

		let url = webhookForm.config.WebHook.url.replaceAll(
			'{notif_msg}',
			'This is a test notification'
		);
		let headers = headersString.split('\n').reduce((res: Record<string, string>, line) => {
			let [key, value] = line.split(':').map((part) => part.trim());
			if (key && value) res[key] = value;
			return res;
		}, {});
		let body = '';

		const options: {
			method: string;
			headers: Record<string, string>;
			body?: string;
		} = {
			method: webhookForm.config.WebHook.method,
			headers: headers
		};

		if (
			['POST', 'PUT', 'PATCH'].includes(webhookForm.config.WebHook.method) &&
			webhookForm.config.WebHook.body
		) {
			body = webhookForm.config.WebHook.body.replaceAll(
				'{notif_msg}',
				'This is a test notification'
			);
			options.body = body;
		}

		fetch(url, options)
			.then((response) => {
				if (!response.ok) throw new Error(`Status: ${response.status}`);
				return response.text();
			})
			.then(() => {
				testResultMessage = 'Test notification sent successfully';
				testResultStatus = 'success';
			})
			.catch((error) => {
				testResultMessage = `Error: ${error.message}`;
				testResultStatus = 'error';
			});
	}

	async function deleteNotificationMethod(id: string) {
		is_loading = true;
		const result = await deleteNotificationMethodApi(id);
		is_loading = false;

		if (result.success) {
			notificationMethods = result.data;
		} else {
			console.error('Error deleting notification method:', result.error);
			alert(`Failed to delete notification method: ${result.error}`);
		}
	}
</script>

{#if is_loading}
	<div class="loading">
		<div class="spinner"></div>
	</div>
{:else}
	<div class="dashboard settings" transition:fade>
		<div class="source-list">
			{#if notificationMethods.length === 0}
				<div class="empty-state">
					<p>No notification methods configured</p>
					<p class="hint">Add one to receive notifications</p>
				</div>
			{:else}
				{#each notificationMethods as method (method.id)}
					<div class="source-item" transition:fly={{ y: 20, duration: 300 }}>
						<div class="source-info">
							<h3>{method.name}</h3>
							<p class="url">{method.config.WebHook.url}</p>
							<span class="method-badge">{method.config.WebHook.method}</span>
						</div>
						<div class="source-actions">
							<button
								class="action-btn toggle"
								onclick={() => {
									webhookForm = { ...method };
									is_new = false;
									toggleDialog();
								}}
							>
								Edit
							</button>
							<button
								class="action-btn delete"
								onclick={() => {
									deleteNotificationMethod(method.id);
								}}
							>
								Delete
							</button>
						</div>
					</div>
				{/each}
			{/if}
		</div>

		<button
			class="add-button"
			onclick={() => {
				is_new = true;
				toggleDialog();
			}}
			aria-label="Add Notification Method"
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				width="24"
				height="24"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<line x1="12" y1="5" x2="12" y2="19"></line>
				<line x1="5" y1="12" x2="19" y2="12"></line>
			</svg>
		</button>

		{#if showDialog}
			<div class="dialog-backdrop" transition:fade={{ duration: 150 }}>
				<div class="dialog">
					<h2>Add Notification Method</h2>
					<p style="margin-top:1rem">Use &lcub;notif_msg&rcub; to insert notification message</p>
					<p></p>
					<p class="hint">Example message: "CPU Usage exceeded 70% for the last 10 minutes"</p>
					<div style="height: 1.5rem"></div>

					<form
						onsubmit={(e) => {
							e.preventDefault();
							addNotificationMethod();
						}}
					>
						<div class="form-group">
							<label for="name">Name</label>
							<input type="text" id="name" bind:value={webhookForm.name} required />
						</div>

						<div class="form-group">
							<label for="url">Webhook URL</label>
							<input
								type="url"
								id="url"
								bind:value={webhookForm.config.WebHook.url}
								required
								placeholder="https://"
							/>
						</div>

						<div class="form-group">
							<label for="method">Method</label>
							<select
								id="method"
								bind:value={webhookForm.config.WebHook.method}
								onchange={() => {
									showBodyTextArea = ['POST', 'PUT', 'PATCH'].includes(
										webhookForm.config.WebHook.method
									);
								}}
							>
								{#each methods as method (method)}
									<option value={method}>{method}</option>
								{/each}
							</select>
						</div>

						<div class="form-group">
							<label for="headers">Headers</label>
							<textarea
								id="headers"
								bind:value={headersString}
								placeholder="Content-type: Application/json"
							></textarea>
						</div>

						{#if showBodyTextArea}
							<div class="form-group">
								<label for="body">Request Body</label>
								<textarea
									id="body"
									bind:value={webhookForm.config.WebHook.body}
									placeholder="message=&lcub;notif_msg&rcub;"
								></textarea>
							</div>
						{/if}

						<div class="dialog-actions">
							<div
								style="display: flex; align-items: center; flex-direction: row; gap: 1rem; max-width:50%;"
							>
								<button type="button" class="test" onclick={sendTestNotification}>
									<svg
										xmlns="http://www.w3.org/2000/svg"
										width="16"
										height="16"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2"
										stroke-linecap="round"
										stroke-linejoin="round"
										class="test-icon"><path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z"></path></svg
									>
									Test
								</button>
								<span
									class="hint"
									style="font-size: 0.8rem; color: {testResultStatus === 'error'
										? '#ef4444'
										: testResultStatus === 'success'
											? '#4ade80'
											: 'inherit'};"
								>
									{testResultMessage}
								</span>
							</div>
							<div
								style="display: flex; justify-content: end; align-items: center; flex-direction: row; gap:0.5rem;"
							>
								<button type="button" class="cancel" onclick={toggleDialog}>Cancel</button>

								<button type="submit" class="submit">Save</button>
							</div>
						</div>
					</form>
				</div>
			</div>
		{/if}
	</div>
{/if}
