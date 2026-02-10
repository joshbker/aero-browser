<script>
	import { onMount } from 'svelte'
	import { listen } from '@tauri-apps/api/event'

	let hoveredUrl = $state('')
	let visible = $derived(hoveredUrl !== '')

	onMount(async () => {
		const unlisten = await listen('link_hover', (event) => {
			hoveredUrl = event.payload.url || ''
		})

		return unlisten
	})
</script>

{#if visible}
	<div class="fixed bottom-0 left-0 max-w-[60%] px-2 py-0.5 bg-neutral-800 border-t border-r border-neutral-700 rounded-tr text-xxs text-neutral-400 truncate z-50">
		{hoveredUrl}
	</div>
{/if}
