<script lang="ts">
	import '$lib/style-settings.css';
	import { cat2names, formatBytes, formatBytesPerSecond, types2names, var2unit } from '$lib/utils';
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
					<div class="source-item" transition:fly={{ y: 20, duration: 300 }}>
						<div class="source-info">
							<h3>
								<span style="padding-right: 0.6rem;">
									{types2names[alert.var.var]}
									{#if alert.var.cat !== 'sys'}
										({alert.var.resrc})
									{/if}
								</span>
								<span style="padding: 0.6rem;">{alert.operator}</span>

								{#if var2unit[alert.var.var] === 'B/s'}
									<span style="padding: 0.6rem;">{formatBytesPerSecond(alert.threshold)}</span>
								{:else if var2unit[alert.var.var] === 'B'}
									<span style="padding: 0.6rem;">{formatBytes(alert.threshold)}</span>
								{:else}
									<span style="padding: 0.6rem;">{alert.threshold}{var2unit[alert.var.var]}</span>
								{/if}
							</h3>
							<span class="status-badge" class:enabled={alert.enabled}>
								{alert.enabled ? 'Active' : 'Inactive'}
							</span>
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
									deleteAlertHandler(alert.id);
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
						<h2 style="margin-bottom:1rem;">Add Alert</h2>

						<form
							onsubmit={(e) => {
								e.preventDefault();
								addAlert();
							}}
						>
							<div class="form-group">
								<label for="notifs">notification method</label>
								<select id="notifs" bind:value={selectedNotifMethod} required>
									{#each notifMethods as method (method.id)}
										<option value={method.id}>{method.name}</option>
									{/each}
								</select>
							</div>

							<div class="form-group">
								<label for="name">Time window (minutes) </label>
								<input
									type="number"
									id="name"
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
									<label for="category">Resource Name</label>
									<select id="category" bind:value={selectedResource} required>
										{#each Array.from(new Set(alertVars
													.filter((v) => v.cat === selectedCategory)
													.map((v) => v.resrc))) as resrc (resrc)}
											<option value={resrc}>{resrc}</option>
										{/each}
									</select>
								</div>

								<div class="form-group">
									<label for="category">Property</label>
									<select id="category" bind:value={selectedProperty} required>
										{#each Array.from(alertVars
												.filter((v) => v.cat === selectedCategory)
												.filter((v) => v.resrc === selectedResource)) as alertVar (alertVar.var)}
											<option value={alertVar.var}>{types2names[alertVar.var]}</option>
										{/each}
									</select>
								</div>
							{:else}
								<div class="form-group">
									<label for="category">Property</label>
									<select id="category" bind:value={selectedProperty} required>
										{#each Array.from(alertVars.filter((v) => v.cat === selectedCategory)) as alertVar (alertVar.var)}
											<option value={alertVar.var}>{types2names[alertVar.var]}</option>
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
									{#if alertForm.var.var && var2unit[alertForm.var.var]}
										({var2unit[alertForm.var.var]})
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
	</div>
{/if}
