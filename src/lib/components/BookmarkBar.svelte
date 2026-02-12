<script>
	import { invoke } from '@tauri-apps/api/core'
	import { bookmarks } from '$lib/stores/bookmarks.js'
	import { Folder, ChevronRight } from 'lucide-svelte'

	function navigateTo(url) {
		if (url) {
			invoke('navigate_to', { url })
		}
	}

	function getFavicon(url) {
		try {
			const parsed = new URL(url)
			return `https://www.google.com/s2/favicons?domain=${parsed.hostname}&sz=16`
		} catch {
			return null
		}
	}
</script>

{#if $bookmarks.showBar}
	<div class="flex items-center gap-0.5 h-7 px-2 bg-neutral-800 border-b border-neutral-700 overflow-hidden">
		{#each $bookmarks.barItems as item}
			{#if item.is_folder}
				<button
					class="flex items-center gap-1 px-2 py-0.5 text-xs text-neutral-400 hover:text-neutral-200 hover:bg-neutral-700 rounded truncate max-w-[150px]"
					title={item.title}
				>
					<Folder size={12} class="shrink-0" />
					<span class="truncate">{item.title}</span>
				</button>
			{:else}
				<button
					onclick={() => navigateTo(item.url)}
					class="flex items-center gap-1 px-2 py-0.5 text-xs text-neutral-400 hover:text-neutral-200 hover:bg-neutral-700 rounded truncate max-w-[150px]"
					title={item.title}
				>
					{#if item.url}
						{@const favicon = getFavicon(item.url)}
						{#if favicon}
							<img src={favicon} alt="" class="w-3 h-3 shrink-0" />
						{/if}
					{/if}
					<span class="truncate">{item.title}</span>
				</button>
			{/if}
		{/each}
		{#if $bookmarks.barItems.length === 0}
			<span class="text-xs text-neutral-600 italic">No bookmarks yet — press Ctrl+D to add one</span>
		{/if}
	</div>
{/if}
