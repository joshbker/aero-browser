<script>
	import { X, Loader2 } from 'lucide-svelte'
	import { invoke } from '@tauri-apps/api/core'

	let { tab, isActive = false, onActivate, onClose, onDragStart, onDragEnter, isDragTarget = false } = $props()

	let isHovered = $state(false)

	let displayTitle = $derived(
		tab.title && tab.title !== 'New Tab'
			? tab.title
			: tab.url
				? tab.url.replace(/^https?:\/\//, '').replace(/\/$/, '')
				: 'New Tab'
	)

	// --- Context menu (native popup window via Rust) ---
	function handleContextMenu(e) {
		e.preventDefault()
		invoke('show_context_menu', {
			x: e.clientX,
			y: e.clientY,
			tabLabel: tab.label,
			items: [
				{ label: 'Duplicate Tab', action: 'duplicate' },
				{ separator: true },
				{ label: 'Close Tab', action: 'close' },
				{ label: 'Close Other Tabs', action: 'close_others' },
				{ label: 'Close Tabs to the Right', action: 'close_to_right' },
			]
		}).catch(console.error)
	}

	// --- Mouse-based drag ---
	function handleMouseDown(e) {
		if (e.button !== 0) return
		if (e.target.closest('button')) return
		onDragStart?.(tab.label, e)
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	role="tab"
	tabindex="0"
	aria-selected={isActive}
	data-tab-label={tab.label}
	class="flex items-center gap-1.5 h-full px-3 min-w-[120px] max-w-[200px] cursor-pointer border-r border-neutral-700 transition-colors {isActive ? 'bg-neutral-700' : 'bg-neutral-800 hover:bg-neutral-700/50'} {isDragTarget ? 'border-l-2 border-l-blue-500' : ''}"
	onclick={onActivate}
	onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') onActivate() }}
	onmouseenter={() => { isHovered = true; onDragEnter?.(tab.label) }}
	onmouseleave={() => (isHovered = false)}
	onmousedown={handleMouseDown}
	oncontextmenu={handleContextMenu}
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
