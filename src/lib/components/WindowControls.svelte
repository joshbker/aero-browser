<script>
	import { Window } from '@tauri-apps/api/window'
	import { Minus, Square, X } from 'lucide-svelte'

	// Get the main window by label (our UI runs in a child webview)
	function getMainWindow() {
		return new Window('main')
	}

	async function handleMinimize() {
		await getMainWindow().minimize()
	}

	async function handleMaximize() {
		const win = getMainWindow()
		const isMax = await win.isMaximized()
		if (isMax) {
			await win.unmaximize()
		} else {
			await win.maximize()
		}
	}

	async function handleClose() {
		await getMainWindow().close()
	}
</script>

<div class="flex items-center h-full">
	<button
		onclick={handleMinimize}
		class="h-full px-3.5 flex items-center justify-center hover:bg-neutral-600 transition-colors"
		aria-label="Minimize"
	>
		<Minus size={14} strokeWidth={1.5} />
	</button>
	<button
		onclick={handleMaximize}
		class="h-full px-3.5 flex items-center justify-center hover:bg-neutral-600 transition-colors"
		aria-label="Maximize"
	>
		<Square size={12} strokeWidth={1.5} />
	</button>
	<button
		onclick={handleClose}
		class="h-full px-3.5 flex items-center justify-center hover:bg-red-500 transition-colors"
		aria-label="Close"
	>
		<X size={14} strokeWidth={1.5} />
	</button>
</div>
