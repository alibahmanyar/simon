<script lang="ts">
	import { formatUptime, formatBytes } from '$lib/utils';
	import { gdata } from '$lib/general_socket.svelte';
	import Chart from '$lib/Chart.svelte';
	import type { Disk } from '$lib/types';

	let disks = $state([] as Disk[] | undefined);
	$effect(() => {
		disks = gdata.data?.disk.disks.filter(
			(disk) => (
				!disk.mount_point.includes('/boot')
			)
		);
	});
</script>

{#if disks && disks.length > 0}
	<div class="card">
		<div id="disk-info" class="info-grid-1">
			{#if disks.length > 0}
				{#each disks as disk, index}
					{@const usedPercentage =
						disk.total_space > 0 ? ((1 - (disk.free_space / disk.total_space)) * 100).toFixed(1) : 0}

					<div class="disk-item">
						<!-- <p class="card-title">{disk.fs || `Disk ${index + 1}`}</p> -->
						<p class="highlight-value">{usedPercentage}%</p>

						<div class="bar">
							<div class="bar-fill" style="width: {usedPercentage}%"></div>
						</div>
						<div>
							{#if disk.mount_point}
								<div class="info-item">
									<span class="info-label">Mount Point:</span>
									<span class="info-value">{disk.mount_point}</span>
								</div>
							{/if}

							{#if disk.fs}
								<div class="info-item">
									<span class="info-label">Filesystem:</span>
									<span class="info-value">{disk.fs}</span>
								</div>
							{/if}

							{#if disk.kind}
								<div class="info-item">
									<span class="info-label">Kind:</span>
									<span class="info-value">{disk.kind}</span>
								</div>
							{/if}

							{#if disk.removable}
								<div class="info-item">
									<span class="info-label">Removable:</span>
									<span class="info-value">{disk.removable}</span>
								</div>
							{/if}
						</div>

						{#if disk.total_space > 0}
							<div class="info-grid">
								<div class="info-item">
									<span class="info-label">Total Space:</span>
									<span class="info-value">{formatBytes(disk.total_space)}</span>
								</div>
								<div class="info-item">
									<span class="info-label">Used Space:</span>
									<span class="info-value">{formatBytes(disk.total_space - disk.free_space)}</span>
								</div>
								<div class="info-item">
									<span class="info-label">Free Space:</span>
									<span class="info-value">{formatBytes(disk.free_space)}</span>
								</div>
							</div>
						{/if}

						{#if disk.io[0] > 0 || disk.io[1] > 0}
							<p class="card-title" style="margin-top: 1rem;">I/O Statistics</p>
							<div class="info-grid">
								<div class="info-item">
									<span class="info-label">Total Read:</span>
									<span class="info-value">{formatBytes(disk.io[0])}</span>
								</div>
								<div class="info-item">
									<span class="info-label">Total Written:</span>
									<span class="info-value">{formatBytes(disk.io[1])}</span>
								</div>
							</div>
						{/if}
					</div>
				{/each}
			{:else}
				<div class="info-item">
					<span>No disk information found</span>
				</div>
			{/if}
		</div>
	</div>
{:else}
	<div class="info-item">
		<span>No disk information found</span>
	</div>
{/if}
