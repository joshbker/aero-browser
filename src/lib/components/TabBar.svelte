<script>
	import { Plus } from 'lucide-svelte'
	import { Window } from '@tauri-apps/api/window'
	import { invoke } from '@tauri-apps/api/core'
	import { listen } from '@tauri-apps/api/event'
	import { onMount } from 'svelte'
	import { tabs } from '$lib/stores/tabs.js'
	import Tab from './Tab.svelte'
	import WindowControls from './WindowControls.svelte'

	let { tabList = [], activeTabLabel = null } = $props()

	// --- Listen for context menu actions from popup window ---
	onMount(async () => {
		const unlisten = await listen('context_menu_action', (event) => {
			const { tab_label, action } = event.payload
			switch (action) {
				case 'duplicate':
					handleDuplicate(tab_label)
					break
				case 'close':
					tabs.close(tab_label)
					break
				case 'close_others':
					handleCloseOthers(tab_label)
					break
				case 'close_to_right':
					handleCloseToRight(tab_label)
					break
			}
		})

		return () => unlisten()
	})

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

	// --- Context menu handlers ---
	async function handleDuplicate(label) {
		try {
			await invoke('tab_duplicate', { label })
		} catch (e) {
			console.error('Failed to duplicate tab:', e)
		}
	}

	async function handleCloseOthers(keepLabel) {
		const toClose = tabList.filter(t => t.label !== keepLabel)
		for (const tab of toClose) {
			await tabs.close(tab.label)
		}
	}

	async function handleCloseToRight(label) {
		const index = tabList.findIndex(t => t.label === label)
		const toClose = tabList.slice(index + 1)
		for (const tab of toClose) {
			await tabs.close(tab.label)
		}
	}

	// --- Mouse-based tab drag reorder ---
	let draggingLabel = $state(null)
	let dragTargetLabel = $state(null)
	const DRAG_THRESHOLD = 5

	function handleTabDragStart(label, e) {
		const startX = e.clientX
		const startY = e.clientY
		let started = false

		function onMouseMove(moveEvent) {
			const dx = moveEvent.clientX - startX
			const dy = moveEvent.clientY - startY
			if (!started && Math.abs(dx) > DRAG_THRESHOLD) {
				started = true
				draggingLabel = label
			}
			if (started) {
				// Find which tab we're hovering over
				const el = document.elementFromPoint(moveEvent.clientX, moveEvent.clientY)
				const tabEl = el?.closest('[data-tab-label]')
				const targetLabel = tabEl?.dataset?.tabLabel || null
				if (targetLabel && targetLabel !== draggingLabel) {
					dragTargetLabel = targetLabel
				}
			}
		}

		function onMouseUp() {
			window.removeEventListener('mousemove', onMouseMove)
			window.removeEventListener('mouseup', onMouseUp)

			if (started && draggingLabel && dragTargetLabel) {
				const targetIndex = tabList.findIndex(t => t.label === dragTargetLabel)
				if (targetIndex >= 0) {
					invoke('tab_reorder', {
						label: draggingLabel,
						newIndex: targetIndex
					}).catch(e => console.error('Failed to reorder tab:', e))
				}
			}

			draggingLabel = null
			dragTargetLabel = null
		}

		window.addEventListener('mousemove', onMouseMove)
		window.addEventListener('mouseup', onMouseUp)
	}

	function handleTabDragEnter(label) {
		if (draggingLabel && label !== draggingLabel) {
			dragTargetLabel = label
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
				onDragStart={handleTabDragStart}
				onDragEnter={handleTabDragEnter}
				isDragTarget={dragTargetLabel === tab.label}
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
