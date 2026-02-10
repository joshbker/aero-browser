<script>
	import { Plus } from 'lucide-svelte'
	import { Window } from '@tauri-apps/api/window'
	import { tabs } from '$lib/stores/tabs.js'
	import Tab from './Tab.svelte'
	import WindowControls from './WindowControls.svelte'

	let { tabList = [], activeTabLabel = null } = $props()

	function isInteractive(e) {
		return e.target.closest('button') || e.target.closest('[role="tab"]')
	}

	function handleMouseDown(e) {
		if (isInteractive(e)) return
		e.preventDefault()

		// e.detail === 2 means the browser detected a double-click
		if (e.detail === 2) {
			const win = new Window('main')
			win.isMaximized().then((isMax) => {
				if (isMax) win.unmaximize()
				else win.maximize()
			})
		} else {
			new Window('main').startDragging()
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="flex items-center h-9 bg-neutral-800 select-none"
	onmousedown={handleMouseDown}
>
	<!-- Tabs -->
	<div class="flex items-center h-full overflow-x-auto flex-1">
		{#each tabList as tab (tab.label)}
			<Tab
				{tab}
				isActive={tab.label === activeTabLabel}
				onActivate={() => tabs.setActive(tab.label)}
				onClose={() => tabs.close(tab.label)}
			/>
		{/each}

		<!-- New tab button -->
		<button
			onclick={() => tabs.create()}
			class="flex items-center justify-center h-full px-2.5 hover:bg-neutral-700 transition-colors"
			aria-label="New tab"
		>
			<Plus size={14} strokeWidth={1.5} class="text-neutral-400" />
		</button>
	</div>

	<!-- Window controls (right side) -->
	<WindowControls />
</div>
