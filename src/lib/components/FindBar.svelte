<script>
	import { X, ChevronUp, ChevronDown } from 'lucide-svelte'
	import { invoke } from '@tauri-apps/api/core'
	import { listen } from '@tauri-apps/api/event'
	import { onMount } from 'svelte'

	let { visible = false, onClose } = $props()

	let query = $state('')
	let inputEl = $state(null)
	let totalMatches = $state(0)
	let currentMatch = $state(0)
	let unlisten = null

	onMount(async () => {
		unlisten = await listen('find_result', (event) => {
			totalMatches = event.payload.total
			currentMatch = event.payload.current
		})

		return () => {
			unlisten?.()
		}
	})

	// Focus input when find bar becomes visible
	// Need a delay because global shortcut fires from content webview context
	// and ui_focus needs time to take effect
	$effect(() => {
		if (visible) {
			const tryFocus = () => {
				inputEl?.focus()
				inputEl?.select()
			}
			// Try multiple times to ensure focus lands
			setTimeout(tryFocus, 50)
			setTimeout(tryFocus, 150)
			setTimeout(tryFocus, 300)
		} else {
			totalMatches = 0
			currentMatch = 0
		}
	})

	let debounceTimer = null

	function handleInput() {
		clearTimeout(debounceTimer)
		if (!query) {
			totalMatches = 0
			currentMatch = 0
			invoke('find_clear').catch(() => {})
			return
		}
		debounceTimer = setTimeout(() => {
			invoke('find_in_page', {
				query,
				forward: true,
				newSearch: true
			}).catch(console.error)
		}, 150)
	}

	async function findNext() {
		if (!query || totalMatches === 0) return
		invoke('find_in_page', {
			query,
			forward: true,
			newSearch: false
		}).catch(console.error)
		// Update counter with wrap-around
		if (currentMatch >= totalMatches) {
			currentMatch = 1
		} else {
			currentMatch++
		}
	}

	async function findPrev() {
		if (!query || totalMatches === 0) return
		invoke('find_in_page', {
			query,
			forward: false,
			newSearch: false
		}).catch(console.error)
		// Update counter with wrap-around
		if (currentMatch <= 1) {
			currentMatch = totalMatches
		} else {
			currentMatch--
		}
	}

	function handleKeyDown(e) {
		if (e.key === 'Enter') {
			e.preventDefault()
			if (e.shiftKey) findPrev()
			else findNext()
		} else if (e.key === 'Escape') {
			onClose?.()
		}
	}

	let matchText = $derived(
		query && totalMatches > 0
			? `${currentMatch}/${totalMatches}`
			: query && totalMatches === 0
				? 'No matches'
				: ''
	)
</script>

{#if visible}
	<div class="absolute right-12 top-[36px] flex items-center gap-1.5 h-10 px-3 bg-neutral-800 border border-neutral-700 rounded-b-md shadow-lg z-50">
		<input
			bind:this={inputEl}
			bind:value={query}
			oninput={handleInput}
			onkeydown={handleKeyDown}
			placeholder="Find in page"
			class="w-48 px-2 py-1 text-sm bg-neutral-700 text-neutral-100 rounded border border-neutral-600 focus:outline-none focus:border-blue-500"
		/>

		{#if matchText}
			<span class="text-xs text-neutral-500 whitespace-nowrap min-w-[60px] text-center">
				{matchText}
			</span>
		{/if}

		<button
			onclick={findPrev}
			disabled={!query || totalMatches === 0}
			class="p-1 rounded text-neutral-400 hover:text-neutral-200 hover:bg-neutral-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
			aria-label="Previous match"
		>
			<ChevronUp size={14} />
		</button>

		<button
			onclick={findNext}
			disabled={!query || totalMatches === 0}
			class="p-1 rounded text-neutral-400 hover:text-neutral-200 hover:bg-neutral-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
			aria-label="Next match"
		>
			<ChevronDown size={14} />
		</button>

		<button
			onclick={onClose}
			class="p-1 rounded text-neutral-400 hover:text-neutral-200 hover:bg-neutral-700 transition-colors"
			aria-label="Close find bar"
		>
			<X size={12} />
		</button>
	</div>
{/if}
