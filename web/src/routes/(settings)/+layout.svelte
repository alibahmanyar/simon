<script>
	import '$lib/style.css';
	let { children } = $props();
    import { page } from '$app/state';
</script>
<h1>Simon</h1>
<div class="dashboard">
	<nav class="tabs">
		<a class="tab" class:active={page.url.pathname==='/notif_methods'} href="/notif_methods">Notification Methods</a>
		<a class="tab" class:active={page.url.pathname==='/alerts'} href="/alerts">Alerts</a>
		<a class="tab home-button" href="/" style="margin-left: auto;" aria-label="Home">
			<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<path d="M5 12h14"></path>
				<path d="M12 5l7 7-7 7"></path>
			</svg>
		</a>
	</nav>
    {@render children()}
</div>


