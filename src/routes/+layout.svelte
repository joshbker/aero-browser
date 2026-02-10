<script>
	import '$lib/styles/app.css'

	let { children } = $props()
</script>

<div class="h-screen w-screen bg-neutral-900 text-neutral-100 flex flex-col overflow-hidden">
	{@render children()}
</div>
