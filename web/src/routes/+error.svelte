<script>
	import { page } from '$app/state';
	import '$lib/style.css';

	if (!page.url.href.endsWith('/')) {
		const url = new URL(page.url.href);
		url.pathname += '/';
		window.location.href = url.href;
	}
</script>
