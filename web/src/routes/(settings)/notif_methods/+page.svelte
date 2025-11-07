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

	// Template system
	let showTemplateSelection = $state(false);
	let selectedTemplate = $state('custom');

	// Template-specific fields
	let telegramBotToken = $state('');
	let telegramChatId = $state('');
	let ntfyTopic = $state('');
	let ntfyServer = $state('https://ntfy.sh');
	let gotifyServer = $state('');
	let gotifyToken = $state('');
	let pushoverUserKey = $state('');
	let pushoverAppToken = $state('');
	let pushbulletAccessToken = $state('');
	let matrixHomeserver = $state('https://matrix.org');
	let matrixRoomId = $state('');
	let matrixAccessToken = $state('');

	interface WebhookTemplate {
		id: string;
		name: string;
		description: string;
		iconSvg: string;
		fields: string[];
	}

	const templates: WebhookTemplate[] = [
		{
			id: 'custom',
			name: 'Custom Webhook',
			description: 'Configure a custom webhook with full control',
			iconSvg:
				'<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgba(255,255,255,0.9)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>',
			fields: []
		},
		{
			id: 'telegram',
			name: 'Telegram',
			description: 'Send notifications to a Telegram chat',
			iconSvg:
				'<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="rgba(255,255,255,0.9)"><path d="M12 0C5.373 0 0 5.373 0 12s5.373 12 12 12 12-5.373 12-12S18.627 0 12 0zm5.562 8.161c-.18 1.897-.962 6.502-1.359 8.627-.168.9-.5 1.201-.82 1.23-.697.064-1.226-.461-1.901-.903-1.056-.692-1.653-1.123-2.678-1.799-1.185-.781-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.139-5.062 3.345-.479.329-.913.489-1.302.481-.428-.009-1.252-.242-1.865-.442-.752-.244-1.349-.374-1.297-.789.027-.216.324-.437.893-.663 3.498-1.524 5.831-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635.099-.002.321.023.465.141.121.098.155.23.171.324.016.093.036.306.02.472z"/></svg>',
			fields: ['botToken', 'chatId']
		},
		{
			id: 'ntfy',
			name: 'ntfy',
			description: 'Simple HTTP-based pub-sub notifications',
			iconSvg:
				'<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgba(255,255,255,0.9)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>',
			fields: ['server', 'topic']
		},
		{
			id: 'gotify',
			name: 'Gotify',
			description: 'Self-hosted push notification service',
			iconSvg:
				'<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgba(255,255,255,0.9)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>',
			fields: ['server', 'token']
		},
		{
			id: 'pushover',
			name: 'Pushover',
			description: 'Simple push notifications',
			iconSvg:
				'<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgba(255,255,255,0.9)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="7" width="20" height="14" rx="2" ry="2"/><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"/></svg>',
			fields: ['userKey', 'appToken']
		},
		{
			id: 'pushbullet',
			name: 'Pushbullet',
			description: 'Send notifications to all your devices',
			iconSvg:
				'<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgba(255,255,255,0.9)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="m8 12 4 4 4-4"/><path d="M12 16V8"/></svg>',
			fields: ['accessToken']
		},
		{
			id: 'matrix',
			name: 'Matrix',
			description: 'Decentralized communication protocol',
			iconSvg:
				'<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="rgba(255,255,255,0.9)"><path d="M.632.55v22.9H2.28V24H0V0h2.28v.55zm7.043 7.26v1.157h.033c.309-.443.683-.784 1.117-1.024.433-.245.936-.365 1.5-.365.54 0 1.033.107 1.481.314.448.208.785.582 1.02 1.108.254-.374.6-.706 1.034-.992.434-.287.95-.43 1.546-.43.453 0 .872.056 1.26.167.388.11.716.286.993.53.276.245.489.559.646.951.152.392.23.863.23 1.417v5.728h-2.349V11.52c0-.286-.01-.559-.032-.812a1.755 1.755 0 0 0-.18-.66 1.106 1.106 0 0 0-.438-.448c-.194-.11-.457-.166-.785-.166-.332 0-.6.064-.803.189a1.38 1.38 0 0 0-.48.499 1.946 1.946 0 0 0-.231.696 5.56 5.56 0 0 0-.06.785v4.768h-2.35v-4.8c0-.254-.004-.503-.018-.752a2.074 2.074 0 0 0-.143-.688 1.052 1.052 0 0 0-.415-.503c-.194-.125-.476-.19-.854-.19-.111 0-.259.024-.439.074-.18.051-.36.143-.53.282-.171.138-.319.337-.439.595-.12.259-.18.6-.18 1.02v4.966H5.46V7.81zm15.693 15.64V.55H21.72V0H24v24h-2.28v-.55z"/></svg>',
			fields: ['homeserver', 'roomId', 'accessToken']
		}
	];

	function applyTemplate() {
		const template = templates.find((t) => t.id === selectedTemplate);
		if (!template) return;

		switch (selectedTemplate) {
			case 'telegram':
				webhookForm.name = 'Telegram Notification';
				webhookForm.config.WebHook.url = `https://api.telegram.org/bot${telegramBotToken}/sendMessage?chat_id=${telegramChatId}&text={notif_msg}`;
				webhookForm.config.WebHook.method = 'GET';
				webhookForm.config.WebHook.body = '';
				headersString = '';
				showBodyTextArea = false;
				break;

			case 'ntfy':
				webhookForm.name = 'ntfy Notification';
				webhookForm.config.WebHook.url = `${ntfyServer}/${ntfyTopic}`;
				webhookForm.config.WebHook.method = 'POST';
				webhookForm.config.WebHook.body = '{notif_msg}';
				headersString = 'Content-Type: text/plain';
				showBodyTextArea = true;
				break;

			case 'gotify':
				webhookForm.name = 'Gotify Notification';
				webhookForm.config.WebHook.url = `${gotifyServer}/message?token=${gotifyToken}`;
				webhookForm.config.WebHook.method = 'POST';
				webhookForm.config.WebHook.body =
					'{"message": "{notif_msg}", "title": "Simon Alert", "priority": 5}';
				headersString = 'Content-Type: application/json';
				showBodyTextArea = true;
				break;

			case 'pushover':
				webhookForm.name = 'Pushover Notification';
				webhookForm.config.WebHook.url = 'https://api.pushover.net/1/messages.json';
				webhookForm.config.WebHook.method = 'POST';
				webhookForm.config.WebHook.body = `{"token": "${pushoverAppToken}", "user": "${pushoverUserKey}", "message": "{notif_msg}"}`;
				headersString = 'Content-Type: application/json';
				showBodyTextArea = true;
				break;

			case 'pushbullet':
				webhookForm.name = 'Pushbullet Notification';
				webhookForm.config.WebHook.url = 'https://api.pushbullet.com/v2/pushes';
				webhookForm.config.WebHook.method = 'POST';
				webhookForm.config.WebHook.body =
					'{"type": "note", "title": "Simon Alert", "body": "{notif_msg}"}';
				headersString = `Access-Token: ${pushbulletAccessToken}\nContent-Type: application/json`;
				showBodyTextArea = true;
				break;

			case 'matrix': {
				webhookForm.name = 'Matrix Notification';
				const txnId = Date.now();
				webhookForm.config.WebHook.url = `${matrixHomeserver}/_matrix/client/v3/rooms/${encodeURIComponent(matrixRoomId)}/send/m.room.message/${txnId}?access_token=${matrixAccessToken}`;
				webhookForm.config.WebHook.method = 'PUT';
				webhookForm.config.WebHook.body = '{"msgtype": "m.text", "body": "{notif_msg}"}';
				headersString = 'Content-Type: application/json';
				showBodyTextArea = true;
				break;
			}
			case 'custom':
			default:
				// Keep existing form values for custom
				break;
		}

		showTemplateSelection = false;
	}

	function resetTemplateFields() {
		telegramBotToken = '';
		telegramChatId = '';
		ntfyTopic = '';
		ntfyServer = 'https://ntfy.sh';
		gotifyServer = '';
		gotifyToken = '';
		pushoverUserKey = '';
		pushoverAppToken = '';
		pushbulletAccessToken = '';
		matrixHomeserver = 'https://matrix.org';
		matrixRoomId = '';
		matrixAccessToken = '';
	}

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
				resetTemplateFields();
				selectedTemplate = 'custom';
				showTemplateSelection = true;
			} else {
				showTemplateSelection = false;
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
				<div class="dialog" style="max-width: 600px;">
					{#if showTemplateSelection}
						<h2>Choose a Template</h2>
						<p class="hint" style="margin-top:0.5rem; margin-bottom: 1.5rem;">
							Select a service to configure notifications quickly
						</p>

						<div class="template-grid">
							{#each templates as template (template.id)}
								<button
									type="button"
									class="template-card"
									class:selected={selectedTemplate === template.id}
									onclick={() => (selectedTemplate = template.id)}
								>
									<!-- eslint-disable-next-line svelte/no-at-html-tags -->
									<span class="template-icon">{@html template.iconSvg}</span>
									<h3>{template.name}</h3>
									<p class="hint">{template.description}</p>
								</button>
							{/each}
						</div>

						{#if selectedTemplate !== 'custom'}
							<div
								style="margin-top: 1.5rem; padding: 1rem; background: rgba(0,0,0,0.2); border-radius: 8px;"
							>
								<h3 style="margin-bottom: 1rem;">
									{templates.find((t) => t.id === selectedTemplate)?.name} Configuration
								</h3>

								{#if selectedTemplate === 'telegram'}
									<div class="form-group">
										<label for="telegram-token">Bot Token</label>
										<input
											type="text"
											id="telegram-token"
											bind:value={telegramBotToken}
											required
											placeholder="123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
										/>
										<p class="hint" style="margin-top: 0.25rem;">
											Get this from @BotFather on Telegram
										</p>
									</div>
									<div class="form-group">
										<label for="telegram-chat">Chat ID</label>
										<input
											type="text"
											id="telegram-chat"
											bind:value={telegramChatId}
											required
											placeholder="-1001234567890"
										/>
										<p class="hint" style="margin-top: 0.25rem;">
											Start a chat with your bot first! Get ID: personal chat (@userinfobot), groups
											(@getidsbot) or
											{#if telegramBotToken}
												<a
													href="https://api.telegram.org/bot{telegramBotToken}/getUpdates"
													target="_blank"
													rel="noopener noreferrer"
													style="color: #4ade80; text-decoration: underline;">check bot updates</a
												>
											{:else}
												<span style="color: rgba(255,255,255,0.4); cursor: not-allowed;"
													>check bot updates</span
												>
											{/if}
										</p>
									</div>
								{:else if selectedTemplate === 'ntfy'}
									<div class="form-group">
										<label for="ntfy-server">Server URL</label>
										<input
											type="url"
											id="ntfy-server"
											bind:value={ntfyServer}
											required
											placeholder="https://ntfy.sh"
										/>
									</div>
									<div class="form-group">
										<label for="ntfy-topic">Topic</label>
										<input
											type="text"
											id="ntfy-topic"
											bind:value={ntfyTopic}
											required
											placeholder="my-server-alerts"
										/>
										<p class="hint" style="margin-top: 0.25rem;">Choose a unique topic name</p>
									</div>
								{:else if selectedTemplate === 'gotify'}
									<div class="form-group">
										<label for="gotify-server">Server URL</label>
										<input
											type="url"
											id="gotify-server"
											bind:value={gotifyServer}
											required
											placeholder="https://gotify.example.com"
										/>
									</div>
									<div class="form-group">
										<label for="gotify-token">Application Token</label>
										<input
											type="text"
											id="gotify-token"
											bind:value={gotifyToken}
											required
											placeholder="AxL..."
										/>
										<p class="hint" style="margin-top: 0.25rem;">
											Create an app in Gotify to get a token
										</p>
									</div>
								{:else if selectedTemplate === 'pushover'}
									<div class="form-group">
										<label for="pushover-user">User Key</label>
										<input
											type="text"
											id="pushover-user"
											bind:value={pushoverUserKey}
											required
											placeholder="uQiRzpo4DXghDmr9QzzfQu27cmVRsG"
										/>
										<p class="hint" style="margin-top: 0.25rem;">Your Pushover user key</p>
									</div>
									<div class="form-group">
										<label for="pushover-token">Application Token</label>
										<input
											type="text"
											id="pushover-token"
											bind:value={pushoverAppToken}
											required
											placeholder="azGDORePK8gMaC0QOYAMyEEuzJnyUi"
										/>
										<p class="hint" style="margin-top: 0.25rem;">
											Create an app in Pushover to get a token
										</p>
									</div>
								{:else if selectedTemplate === 'pushbullet'}
									<div class="form-group">
										<label for="pushbullet-token">Access Token</label>
										<input
											type="password"
											id="pushbullet-token"
											bind:value={pushbulletAccessToken}
											required
											placeholder="o.xxxxxxxxxxxxxxxxxxxx"
										/>
										<p class="hint" style="margin-top: 0.25rem;">
											Get your access token from Pushbullet Settings → Account
										</p>
									</div>
								{:else if selectedTemplate === 'matrix'}
									<div class="form-group">
										<label for="matrix-homeserver">Homeserver URL</label>
										<input
											type="url"
											id="matrix-homeserver"
											bind:value={matrixHomeserver}
											required
											placeholder="https://matrix.org"
										/>
										<p class="hint" style="margin-top: 0.25rem;">Your Matrix homeserver URL</p>
									</div>
									<div class="form-group">
										<label for="matrix-room">Room ID</label>
										<input
											type="text"
											id="matrix-room"
											bind:value={matrixRoomId}
											required
											placeholder="!AbCdEfGhIjKlMnOpQr:matrix.org"
										/>
										<p class="hint" style="margin-top: 0.25rem;">
											Find in Room Settings → Advanced → Internal Room ID
										</p>
									</div>
									<div class="form-group">
										<label for="matrix-token">Access Token</label>
										<input
											type="password"
											id="matrix-token"
											bind:value={matrixAccessToken}
											required
											placeholder="syt_..."
										/>
										<p class="hint" style="margin-top: 0.25rem;">
											Get from Settings → Help & About → Advanced (at bottom)
										</p>
									</div>
								{/if}

								<div style="display: flex; justify-content: end; gap: 0.5rem; margin-top: 1rem;">
									<button type="button" class="action-btn cancel" onclick={toggleDialog}
										>Cancel</button
									>
									<button type="button" class="action-btn submit" onclick={applyTemplate}
										>Apply Template</button
									>
								</div>
							</div>
						{:else}
							<div style="display: flex; justify-content: end; gap: 0.5rem; margin-top: 1.5rem;">
								<button type="button" class="action-btn cancel" onclick={toggleDialog}
									>Cancel</button
								>
								<button type="button" class="action-btn submit" onclick={applyTemplate}
									>Continue</button
								>
							</div>
						{/if}
					{:else}
						<h2>{is_new ? 'Add' : 'Edit'} Notification Method</h2>
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
									style="display: flex; align-items: center; flex-direction: row; gap: 1rem; max-width:45%;"
								>
									<button type="button" class="action-btn test" onclick={sendTestNotification}>
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
									{#if is_new}
										<button
											type="button"
											class="action-btn cancel"
											onclick={() => (showTemplateSelection = true)}>Back</button
										>
									{/if}
									<button type="button" class="action-btn cancel" onclick={toggleDialog}
										>Cancel</button
									>

									<button type="submit" class="action-btn submit">Save</button>
								</div>
							</div>
						</form>
					{/if}
				</div>
			</div>
		{/if}
	</div>
{/if}
