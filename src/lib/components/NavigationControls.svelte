<script>
	import { invoke } from '@tauri-apps/api/core'
	import { ArrowLeft, ArrowRight, RotateCw } from 'lucide-svelte'

	let { isLoading = false } = $props()

	async function handleBack() {
		try {
			await invoke('navigate_back')
		} catch (e) {
			console.error('navigate_back failed:', e)
		}
	}

	async function handleForward() {
		try {
			await invoke('navigate_forward')
		} catch (e) {
			console.error('navigate_forward failed:', e)
		}
	}

	async function handleRefresh() {
		try {
			if (isLoading) {
				await invoke('navigate_stop')
			} else {
				await invoke('navigate_refresh')
			}
		} catch (e) {
			console.error('navigate_refresh/stop failed:', e)
		}
	}
</script>

<div class="flex items-center gap-0.5">
	<button
		onclick={handleBack}
		class="p-1.5 rounded hover:bg-neutral-700 transition-colors text-neutral-400 hover:text-neutral-200"
		aria-label="Go back"
	>
		<ArrowLeft size={16} strokeWidth={1.5} />
	</button>
	<button
		onclick={handleForward}
		class="p-1.5 rounded hover:bg-neutral-700 transition-colors text-neutral-400 hover:text-neutral-200"
		aria-label="Go forward"
	>
		<ArrowRight size={16} strokeWidth={1.5} />
	</button>
	<button
		onclick={handleRefresh}
		class="p-1.5 rounded hover:bg-neutral-700 transition-colors text-neutral-400 hover:text-neutral-200 {isLoading ? 'animate-spin' : ''}"
		aria-label={isLoading ? 'Stop loading' : 'Refresh'}
	>
		<RotateCw size={16} strokeWidth={1.5} />
	</button>
</div>
