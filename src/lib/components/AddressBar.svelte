<script>
	import { invoke } from '@tauri-apps/api/core'
	import { Lock, Globe } from 'lucide-svelte'
	import { resolveInput } from '$lib/utils/url.js'

	let { url = '', isLoading = false } = $props()

	let inputValue = $state('')
	let isFocused = $state(false)
	let inputEl = $state(null)

	// Sync input value with active tab URL when not focused
	$effect(() => {
		if (!isFocused) {
			inputValue = url || ''
		}
	})

	let isHttps = $derived(url?.startsWith('https://'))

	async function handleSubmit(e) {
		e.preventDefault()
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
		isFocused = false
	}

	function handleKeydown(e) {
		if (e.key === 'Escape') {
			inputValue = url || ''
			inputEl?.blur()
		}
	}

	// Expose focus method for keyboard shortcuts
	export function focus() {
		inputEl?.focus()
	}
</script>

<form onsubmit={handleSubmit} class="flex-1 flex items-center">
	<div class="flex items-center gap-2 flex-1 h-7 px-3 bg-neutral-900 rounded-full border border-neutral-700 focus-within:border-blue-500 transition-colors">
		<!-- Security icon -->
		<div class="shrink-0 text-neutral-500">
			{#if isHttps}
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
			placeholder="Search or enter URL"
			spellcheck="false"
			autocomplete="off"
			class="flex-1 bg-transparent text-sm text-neutral-200 placeholder-neutral-500 outline-none"
		/>
	</div>
</form>
