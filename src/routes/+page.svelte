<script>
	import { onMount } from 'svelte'
	import { invoke } from '@tauri-apps/api/core'
	import { register, unregisterAll } from '@tauri-apps/plugin-global-shortcut'
	import { tabs, activeTab } from '$lib/stores/tabs.js'
	import TabBar from '$lib/components/TabBar.svelte'
	import NavigationControls from '$lib/components/NavigationControls.svelte'
	import AddressBar from '$lib/components/AddressBar.svelte'
	import StatusBar from '$lib/components/StatusBar.svelte'

	let addressBar = $state(null)

	onMount(async () => {
		await tabs.init()

		// Register global shortcuts (work even when content webview has focus)
		try {
			await register('CommandOrControl+T', () => {
				tabs.create()
			})
			await register('CommandOrControl+W', () => {
				const state = getActiveTabSync()
				if (state) tabs.close(state.label)
			})
			await register('CommandOrControl+L', () => {
				addressBar?.focus()
			})
			await register('CommandOrControl+R', () => {
				invoke('navigate_refresh')
			})
			await register('F5', () => {
				invoke('navigate_refresh')
			})
			await register('CommandOrControl+Shift+T', () => {
				// TODO: reopen last closed tab
				tabs.create()
			})
			await register('CommandOrControl+Tab', () => {
				tabs.activateNext()
			})
			await register('CommandOrControl+Shift+Tab', () => {
				tabs.activatePrevious()
			})
			await register('Alt+Left', () => {
				invoke('navigate_back')
			})
			await register('Alt+Right', () => {
				invoke('navigate_forward')
			})
			for (let i = 1; i <= 9; i++) {
				await register(`CommandOrControl+${i}`, ((index) => () => {
					tabs.activateByIndex(index)
				})(i))
			}
		} catch (e) {
			console.error('Failed to register global shortcuts:', e)
		}

		// Also keep local keyboard shortcuts for things like Ctrl+L
		// that need DOM interaction (focus address bar)
		function handleKeydown(e) {
			if (e.ctrlKey && e.key === 'l') {
				e.preventDefault()
				addressBar?.focus()
			}
		}

		window.addEventListener('keydown', handleKeydown)

		return async () => {
			window.removeEventListener('keydown', handleKeydown)
			try {
				await unregisterAll()
			} catch (e) {
				// Ignore cleanup errors
			}
		}
	})

	// Helper to synchronously get active tab from store
	function getActiveTabSync() {
		let result = null
		const unsub = activeTab.subscribe((tab) => {
			result = tab
		})
		unsub()
		return result
	}
</script>

<div class="flex-1 flex flex-col">
	<!-- Tab bar (includes window controls) -->
	<TabBar tabList={$tabs.tabs} activeTabLabel={$tabs.activeTabLabel} />

	<!-- Toolbar -->
	<div class="flex items-center gap-2 h-10 px-2 bg-neutral-800 border-b border-neutral-700">
		<NavigationControls
			isLoading={$activeTab?.is_loading || false}
			canGoBack={$activeTab?.can_go_back || false}
			canGoForward={$activeTab?.can_go_forward || false}
		/>
		<AddressBar
			bind:this={addressBar}
			url={$activeTab?.url || ''}
			isLoading={$activeTab?.is_loading || false}
		/>
	</div>

	<StatusBar />
</div>
