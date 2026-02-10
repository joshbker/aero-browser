<script>
	import { X, Loader2 } from 'lucide-svelte'

	let { tab, isActive = false, onActivate, onClose } = $props()

	let isHovered = $state(false)

	let displayTitle = $derived(
		tab.title && tab.title !== 'New Tab'
			? tab.title
			: tab.url
				? tab.url.replace(/^https?:\/\//, '').replace(/\/$/, '')
				: 'New Tab'
	)
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	role="tab"
	tabindex="0"
	aria-selected={isActive}
	class="flex items-center gap-1.5 h-full px-3 min-w-[120px] max-w-[200px] cursor-pointer border-r border-neutral-700 transition-colors {isActive ? 'bg-neutral-700' : 'bg-neutral-800 hover:bg-neutral-700/50'}"
	onclick={onActivate}
	onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') onActivate() }}
	onmouseenter={() => (isHovered = true)}
	onmouseleave={() => (isHovered = false)}
>
	{#if tab.is_loading}
		<div class="shrink-0 animate-spin text-neutral-400">
			<Loader2 size={12} />
		</div>
	{:else if tab.favicon}
		<img src={tab.favicon} alt="" class="shrink-0 w-3 h-3 rounded-sm" />
	{:else}
		<div class="shrink-0 w-3 h-3 rounded-sm bg-neutral-600"></div>
	{/if}

	<span class="flex-1 text-xxs truncate text-neutral-300 select-none">
		{displayTitle}
	</span>

	{#if isHovered || isActive}
		<button
			onclick={(e) => {
				e.stopPropagation()
				onClose()
			}}
			class="shrink-0 p-0.5 rounded hover:bg-neutral-500 transition-colors"
			aria-label="Close tab"
		>
			<X size={10} />
		</button>
	{/if}
</div>
