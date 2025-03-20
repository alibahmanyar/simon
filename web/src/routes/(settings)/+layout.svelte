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
		<a class="tab home-button" href="/" style="margin-left: auto;">Stats</a>
	</nav>
    {@render children()}
</div>


