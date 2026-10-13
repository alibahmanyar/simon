<script lang="ts">
	import '$lib/style-settings.css';
	import {
		cat2names,
		formatBytes,
		formatBytesPerSecond,
		types2names,
		var2unit
	} from '$lib/utils.svelte';
	import {
		getAlerts,
		getAlertVars,
		getNotificationMethods,
		saveAlert,
		deleteAlert as deleteAlertApi
	} from '$lib/api';
	import { onMount } from 'svelte';
	import { fly, fade } from 'svelte/transition';
	import type { Alert, AlertVar, NotificationMethod } from '$lib/types';

	let is_loading: boolean = $state(true);

	let showDialog = $state(false);
	let is_new = $state(true);

	let selectedCategory = $state('');
	let selectedResource = $state('');
	let selectedProperty = $state('');

	let selectedNotifMethod = $state('');

	// Form data
	let alertForm: Alert = $state({
		id: '-1',
		var: {
			cat: '',
			var: '',
			resrc: ''
		},
		threshold: 0,
		operator: '',
		time_window: 1,
		enabled: true,
		firing: false,
		notif_methods: []
	});

	let alerts: Alert[] = $state([]);
	let alertVars: AlertVar[] = $state([]);

	let notifMethods: NotificationMethod[] = $state([]);

	onMount(async () => {
		is_loading = true;
		try {
			// Fetch all data in parallel
			const [alertsResult, alertVarsResult, notifMethodsResult] = await Promise.all([
				getAlerts(),
				getAlertVars(),
				getNotificationMethods()
			]);

			// Handle alerts
			if (alertsResult.success) {
				alerts = alertsResult.data;
			} else {
				console.error('Failed to fetch alerts:', alertsResult.error);
			}

			// Handle alert vars
			if (alertVarsResult.success) {
				alertVars = alertVarsResult.data;
			} else {
				console.error('Failed to fetch alert vars:', alertVarsResult.error);
			}

			// Handle notification methods
			if (notifMethodsResult.success) {
				notifMethods = notifMethodsResult.data;
			} else {
				console.error('Failed to fetch notification methods:', notifMethodsResult.error);
			}
		} catch (error) {
			console.error('Error fetching data:', error);
		} finally {
			is_loading = false;
		}
	});

	function toggleDialog() {
		showDialog = !showDialog;
		if (showDialog) {
			if (is_new) {
				// Reset form when opening
				alertForm = {
					id: '-1',
					var: {
						cat: '',
						var: '',
						resrc: ''
					},
					threshold: 0,
					operator: '',
					time_window: 1,
					enabled: true,
					firing: false,
					notif_methods: []
				};
				selectedCategory = '';
				selectedResource = '';
				selectedProperty = '';
				selectedNotifMethod = '';
			}
		}
	}

	async function addAlert() {
		alertForm.notif_methods = [selectedNotifMethod];
		alertForm.var.cat = selectedCategory;
		alertForm.var.resrc = selectedResource;
		alertForm.var.var = selectedProperty;

		is_loading = true;
		const result = await saveAlert(alertForm);
		is_loading = false;

		if (result.success) {
			alerts = result.data;
			toggleDialog();
		} else {
			console.error('Failed to add alert:', result.error);
		}
	}

	function alertSubject(alert: Alert) {
		const name = types2names[alert.var.var];
		return alert.var.cat !== 'sys' ? `${name} (${alert.var.resrc})` : name;
	}

	function alertCondition(alert: Alert) {
		const unit = var2unit[alert.var.var];
		let value;
		if (unit === 'B/s') value = formatBytesPerSecond(alert.threshold);
		else if (unit === 'B') value = formatBytes(alert.threshold);
		else value = `${alert.threshold}${unit}`;
		return `${alert.operator === '<' ? 'below' : 'above'} ${value}`;
	}

	function formatMinutes(minutes: number) {
		if (minutes < 60) return `${minutes} min`;
		const h = Math.floor(minutes / 60);
		const m = minutes % 60;
		return m ? `${h} h ${m} min` : `${h} h`;
	}

	function methodNames(alert: Alert) {
		return alert.notif_methods
			.map((id) => notifMethods.find((m) => m.id === id)?.name)
			.filter((name) => name !== undefined);
	}

	let pendingDelete: Alert | null = $state(null);

	async function confirmDelete() {
		if (!pendingDelete) return;
		const id = pendingDelete.id;
		pendingDelete = null;
		await deleteAlertHandler(id);
	}

	async function deleteAlertHandler(id: string) {
		is_loading = true;
		const result = await deleteAlertApi(id);
		is_loading = false;

		if (result.success) {
			alerts = result.data;
		} else {
			console.error('Failed to delete alert:', result.error);
		}
	}
</script>

<svelte:window
	onkeydown={(e) => {
		if (e.key === 'Escape' && pendingDelete) pendingDelete = null;
	}}
/>

{#if is_loading}
	<div class="loading">
		<div class="spinner"></div>
	</div>
{:else}
	<div class="dashboard settings" transition:fade>
		<div class="source-list">
			{#if alerts.length === 0}
				<div class="empty-state">
					<p>No alerts configured</p>
					<p class="hint">Add one to receive alerts</p>
				</div>
			{:else}
				{#each alerts as alert (alert.id)}
					{@const firing = alert.enabled && alert.firing}
					{@const methods = methodNames(alert)}
					<div class="source-item" class:firing transition:fly={{ y: 20, duration: 300 }}>
						<div class="source-info">
							<h3 class="alert-title">
								{alertSubject(alert)}
								<span class="alert-condition">{alertCondition(alert)}</span>
							</h3>
							<div class="alert-meta">
								{#if firing}
									<span class="status-badge firing">Firing</span>
								{:else}
									<span class="status-badge" class:enabled={alert.enabled}>
										{alert.enabled ? 'Active' : 'Inactive'}
									</span>
								{/if}
								<span>
									for {formatMinutes(alert.time_window)} ·
									{#if methods.length > 0}
										via {methods.join(', ')}
									{:else}
										<span class="alert-warning">No notification method</span>
									{/if}
								</span>
							</div>
						</div>
						<div class="source-actions">
							<button
								class="action-btn toggle"
								onclick={() => {
									alertForm = { ...alert };
									selectedCategory = alert.var.cat;
									selectedResource = alert.var.resrc;
									selectedProperty = alert.var.var;
									selectedNotifMethod = alert.notif_methods[0];
									is_new = false;
									toggleDialog();
								}}
							>
								Edit
							</button>
							<button
								class="action-btn delete"
								onclick={() => {
									pendingDelete = alert;
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
			aria-label="Add Alert"
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
					{#if notifMethods.length > 0}
						<h2 style="margin-bottom:1rem;">{is_new ? 'Add' : 'Edit'} Alert</h2>

						<form
							onsubmit={(e) => {
								e.preventDefault();
								addAlert();
							}}
						>
							<div class="form-group">
								<label for="notifs">Notification method</label>
								<select id="notifs" bind:value={selectedNotifMethod} required>
									{#each notifMethods as method (method.id)}
										<option value={method.id}>{method.name}</option>
									{/each}
								</select>
							</div>

							<div class="form-group">
								<label for="time_window">Time window (minutes)</label>
								<input
									type="number"
									id="time_window"
									min="1"
									bind:value={alertForm.time_window}
									required
								/>
							</div>

							<div class="form-group">
								<label for="category">Category</label>
								<select id="category" bind:value={selectedCategory} required>
									{#each new Set(alertVars.map((v) => v.cat)) as cat (cat)}
										<option value={cat}>{cat2names[cat]}</option>
									{/each}
								</select>
							</div>

							{#if selectedCategory != 'sys'}
								<div class="form-group">
									<label for="resource">Resource Name</label>
									<select id="resource" bind:value={selectedResource} required>
										{#each Array.from(new Set(alertVars
													.filter((v) => v.cat === selectedCategory)
													.map((v) => v.resrc))) as resrc (resrc)}
											<option value={resrc}>{resrc}</option>
										{/each}
									</select>
								</div>

								<div class="form-group">
									<label for="property">Property</label>
									<select id="property" bind:value={selectedProperty} required>
										{#each Array.from(alertVars
												.filter((v) => v.cat === selectedCategory)
												.filter((v) => v.resrc === selectedResource)) as alertVar (alertVar.var)}
											<option value={alertVar.var}
												>{types2names[alertVar.var]} ({var2unit[alertVar.var]})</option
											>
										{/each}
									</select>
								</div>
							{:else}
								<div class="form-group">
									<label for="property">Property</label>
									<select id="property" bind:value={selectedProperty} required>
										{#each Array.from(alertVars.filter((v) => v.cat === selectedCategory)) as alertVar (alertVar.var)}
											<option value={alertVar.var}
												>{types2names[alertVar.var]} ({var2unit[alertVar.var]})</option
											>
										{/each}
									</select>
								</div>
							{/if}

							<div class="form-group">
								<label for="condition">Condition</label>
								<select id="condition" bind:value={alertForm.operator} required>
									<option value=">">&gt;</option>
									<option value="<">&lt;</option>
								</select>
							</div>

							<div class="form-group">
								<label for="threshold"
									>Threshold
									{#if selectedProperty && var2unit[selectedProperty]}
										({var2unit[selectedProperty]})
									{/if}
								</label>
								<input
									type="number"
									step="0.1"
									id="threshold"
									bind:value={alertForm.threshold}
									required
								/>
							</div>

							<label class="switch-label">
								<span>Active:</span>
								<label class="switch">
									<input type="checkbox" id="alertEnabled" bind:checked={alertForm.enabled} />
									<span class="slider"></span>
								</label>
							</label>

							<div class="dialog-actions">
								<div
									style="width: 100%; display: flex; justify-content: end; align-items: center; flex-direction: row; gap:0.5rem;"
								>
									<button type="button" class="cancel" onclick={toggleDialog}>Cancel</button>

									<button type="submit" class="submit">Save</button>
								</div>
							</div>
						</form>
					{:else}
						<div>No notification methods found; please add one first</div>
						<div class="dialog-actions">
							<div
								style="width: 100%; display: flex; justify-content: end; align-items: center; flex-direction: row; gap:0.5rem;"
							>
								<button type="button" class="cancel" onclick={toggleDialog}>OK</button>
							</div>
						</div>
					{/if}
				</div>
			</div>
		{/if}

		{#if pendingDelete}
			<div class="dialog-backdrop" transition:fade={{ duration: 150 }}>
				<div
					class="dialog dialog-sm"
					role="alertdialog"
					aria-modal="true"
					aria-labelledby="delete-title"
					aria-describedby="delete-desc"
				>
					<h2 id="delete-title">Delete alert?</h2>
					<p id="delete-desc" class="dialog-text">
						<strong>{alertSubject(pendingDelete)} {alertCondition(pendingDelete)}</strong> will be permanently
						removed.
					</p>
					<div class="dialog-actions dialog-actions-end">
						<button type="button" class="cancel" onclick={() => (pendingDelete = null)}>
							Cancel
						</button>
						<button type="button" class="danger" onclick={confirmDelete}>Delete</button>
					</div>
				</div>
			</div>
		{/if}
	</div>
{/if}
