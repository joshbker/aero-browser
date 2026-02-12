<script>
	import { invoke } from '@tauri-apps/api/core'
	import { Lock, Globe, Settings } from 'lucide-svelte'
	import { resolveInput, isAeroUrl } from '$lib/utils/url.js'
	import { history } from '$lib/stores/history.js'

	let { url = '', isLoading = false } = $props()

	let inputValue = $state('')
	let isFocused = $state(false)
	let inputEl = $state(null)
	let suggestions = $state([])
	let selectedIndex = $state(-1)
	let debounceTimer = $state(null)

	// Sync input value with active tab URL when not focused
	$effect(() => {
		if (!isFocused) {
			inputValue = url || ''
		}
	})

	let isHttps = $derived(url?.startsWith('https://'))
	let isInternal = $derived(isAeroUrl(url || ''))

	async function fetchSuggestions(query) {
		if (!query || query.length < 2) {
			suggestions = []
			return
		}
		try {
			suggestions = await history.suggest(query, 6)
		} catch {
			suggestions = []
		}
	}

	function handleInput() {
		selectedIndex = -1
		clearTimeout(debounceTimer)
		debounceTimer = setTimeout(() => fetchSuggestions(inputValue), 150)
	}

	async function handleSubmit(e) {
		e.preventDefault()
		suggestions = []

		// If a suggestion is selected, use its URL
		if (selectedIndex >= 0 && selectedIndex < suggestions.length) {
			const selected = suggestions[selectedIndex]
			try {
				await invoke('navigate_to', { url: selected.url })
				inputEl?.blur()
			} catch (err) {
				console.error('Navigation failed:', err)
			}
			return
		}

		const resolved = resolveInput(inputValue)
		if (!resolved) return

		try {
			await invoke('navigate_to', { url: resolved })
			inputEl?.blur()
		} catch (err) {
			console.error('Navigation failed:', err)
		}
	}

	function handleFocus() {
		isFocused = true
		// Select all text on focus
		setTimeout(() => inputEl?.select(), 0)
	}

	function handleBlur() {
		// Delay to allow suggestion click to fire
		setTimeout(() => {
			isFocused = false
			suggestions = []
			selectedIndex = -1
		}, 150)
	}

	function handleKeydown(e) {
		if (e.key === 'Escape') {
			if (suggestions.length > 0) {
				suggestions = []
				selectedIndex = -1
			} else {
				inputValue = url || ''
				inputEl?.blur()
			}
		} else if (e.key === 'ArrowDown') {
			e.preventDefault()
			if (suggestions.length > 0) {
				selectedIndex = Math.min(selectedIndex + 1, suggestions.length - 1)
			}
		} else if (e.key === 'ArrowUp') {
			e.preventDefault()
			if (suggestions.length > 0) {
				selectedIndex = Math.max(selectedIndex - 1, -1)
			}
		}
	}

	async function selectSuggestion(item) {
		suggestions = []
		selectedIndex = -1
		try {
			await invoke('navigate_to', { url: item.url })
			inputEl?.blur()
		} catch (err) {
			console.error('Navigation failed:', err)
		}
	}

	// Expose focus method for keyboard shortcuts
	export function focus() {
		inputEl?.focus()
	}
</script>

<form onsubmit={handleSubmit} class="flex-1 flex items-center relative">
	<div class="flex items-center gap-2 flex-1 h-7 px-3 bg-neutral-900 rounded-full border border-neutral-700 focus-within:border-blue-500 transition-colors">
		<!-- Security icon -->
		<div class="shrink-0 text-neutral-500">
			{#if isInternal}
				<Settings size={12} />
			{:else if isHttps}
				<Lock size={12} />
			{:else}
				<Globe size={12} />
			{/if}
		</div>

		<!-- URL input -->
		<input
			bind:this={inputEl}
			bind:value={inputValue}
			onfocus={handleFocus}
			onblur={handleBlur}
			onkeydown={handleKeydown}
			oninput={handleInput}
			placeholder="Search or enter URL"
			spellcheck="false"
			autocomplete="off"
			class="flex-1 bg-transparent text-sm text-neutral-200 placeholder-neutral-500 outline-none"
		/>
	</div>

	<!-- Autocomplete suggestions -->
	{#if suggestions.length > 0 && isFocused}
		<div class="absolute top-full left-0 right-0 mt-1 bg-neutral-800 border border-neutral-700 rounded shadow-lg z-50 overflow-hidden">
			{#each suggestions as item, i}
				<button
					type="button"
					onmousedown={() => selectSuggestion(item)}
					class="flex flex-col w-full px-3 py-1.5 text-left hover:bg-neutral-700 transition-colors
						{i === selectedIndex ? 'bg-neutral-700' : ''}"
				>
					<span class="text-sm text-neutral-200 truncate">{item.title || item.url}</span>
					<span class="text-xs text-neutral-500 truncate">{item.url}</span>
				</button>
			{/each}
		</div>
	{/if}
</form>
