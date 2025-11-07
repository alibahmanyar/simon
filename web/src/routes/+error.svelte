<script>
	import { page } from '$app/state';
	import '$lib/style.css';

	if (!page.url.href.endsWith('/')) {
		const url = new URL(page.url.href);
		url.pathname += '/';
		window.location.href = url.href;
	}

	function goBack() {
		window.history.go(-1);
	}
</script>

<div class="error-container">
	<div class="error-content">
		<h1 class="error-code">{page.status}</h1>
		<h2 class="error-message">
			{#if page.status === 404}
				Page Not Found
			{:else if page.status === 500}
				Internal Server Error
			{:else}
				Error
			{/if}
		</h2>
		<p class="error-description">
			{#if page.status === 404}
				The page you're looking for doesn't exist or has been moved.
			{:else}
				{page.error?.message || 'Something went wrong. Please try again later.'}
			{/if}
		</p>
		<button onclick={goBack} class="home-link">Go Back</button>
	</div>
</div>

<style>
	.home-link {
		display: inline-block;
		padding: 0.75rem 1.5rem;
		background-color: #007bff;
		color: white;
		text-decoration: none;
		border-radius: 0.375rem;
		border: none;
		font-size: 1rem;
		font-weight: 500;
		cursor: pointer;
		transition: background-color 0.2s ease;
	}

	.home-link:hover {
		background-color: #0056b3;
	}

	.home-link:active {
		background-color: #004494;
	}

	.home-link:focus {
		outline: 2px solid #007bff;
		outline-offset: 2px;
	}
</style>
