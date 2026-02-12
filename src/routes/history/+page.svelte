<script>
	import { onMount } from 'svelte'
	import { invoke } from '@tauri-apps/api/core'
	import { history } from '$lib/stores/history.js'
	import { Search, Trash2, Clock, X } from 'lucide-svelte'

	let searchQuery = $state('')
	let entries = $state([])
	let loaded = $state(false)
	let showClearDialog = $state(false)

	onMount(async () => {
		entries = await history.getRecent(200)
		loaded = true
	})

	async function handleSearch() {
		if (searchQuery.trim()) {
			entries = await history.search(searchQuery)
		} else {
			entries = await history.getRecent(200)
		}
	}

	async function handleDelete(id) {
		await history.delete(id)
		entries = entries.filter((e) => e.id !== id)
	}

	async function handleClear(timeframe) {
		await history.clear(timeframe)
		entries = await history.getRecent(200)
		showClearDialog = false
	}

	function navigateTo(url) {
		invoke('navigate_to', { url })
	}

	function groupByDate(items) {
		const groups = {}
		for (const item of items) {
			const date = new Date(item.last_visited + 'Z')
			const today = new Date()
			const yesterday = new Date(today)
			yesterday.setDate(yesterday.getDate() - 1)

			let label
			if (date.toDateString() === today.toDateString()) {
				label = 'Today'
			} else if (date.toDateString() === yesterday.toDateString()) {
				label = 'Yesterday'
			} else {
				label = date.toLocaleDateString(undefined, {
					weekday: 'long',
					year: 'numeric',
					month: 'long',
					day: 'numeric',
				})
			}

			if (!groups[label]) groups[label] = []
			groups[label].push(item)
		}
		return Object.entries(groups)
	}

	let grouped = $derived(groupByDate(entries))
</script>

{#if !loaded}
	<div class="flex items-center justify-center h-full text-neutral-500">
		Loading history...
	</div>
{:else}
	<div class="flex flex-col h-full bg-neutral-900 text-neutral-200">
		<!-- Header -->
		<div class="flex items-center justify-between p-6 pb-4">
			<h1 class="text-lg font-semibold">History</h1>
			<button
				onclick={() => showClearDialog = !showClearDialog}
				class="px-3 py-1.5 text-sm bg-neutral-800 border border-neutral-700 rounded hover:bg-neutral-700 transition-colors"
			>
				Clear browsing data
			</button>
		</div>

		<!-- Clear dialog -->
		{#if showClearDialog}
			<div class="mx-6 mb-4 p-4 bg-neutral-800 border border-neutral-700 rounded">
				<p class="text-sm text-neutral-400 mb-3">Clear browsing history:</p>
				<div class="flex gap-2">
					<button onclick={() => handleClear('hour')} class="px-3 py-1 text-xs bg-neutral-700 rounded hover:bg-neutral-600">Last hour</button>
					<button onclick={() => handleClear('day')} class="px-3 py-1 text-xs bg-neutral-700 rounded hover:bg-neutral-600">Last 24 hours</button>
					<button onclick={() => handleClear('week')} class="px-3 py-1 text-xs bg-neutral-700 rounded hover:bg-neutral-600">Last 7 days</button>
					<button onclick={() => handleClear('all')} class="px-3 py-1 text-xs bg-red-900/50 text-red-300 rounded hover:bg-red-900/80">All time</button>
					<button onclick={() => showClearDialog = false} class="px-3 py-1 text-xs bg-neutral-700 rounded hover:bg-neutral-600 ml-auto">Cancel</button>
				</div>
			</div>
		{/if}

		<!-- Search -->
		<div class="px-6 pb-4">
			<div class="flex items-center gap-2 h-8 px-3 bg-neutral-800 border border-neutral-700 rounded focus-within:border-blue-500">
				<Search size={14} class="text-neutral-500 shrink-0" />
				<input
					bind:value={searchQuery}
					oninput={handleSearch}
					placeholder="Search history"
					class="flex-1 bg-transparent text-sm text-neutral-200 placeholder-neutral-500 outline-none"
				/>
			</div>
		</div>

		<!-- Results -->
		<div class="flex-1 overflow-y-auto px-6">
			{#if entries.length === 0}
				<p class="text-neutral-500 text-sm">
					{searchQuery ? 'No results found.' : 'No browsing history yet.'}
				</p>
			{:else}
				{#each grouped as [dateLabel, items]}
					<div class="mb-6">
						<h2 class="text-xs font-medium text-neutral-500 uppercase tracking-wide mb-2">{dateLabel}</h2>
						<div class="space-y-0.5">
							{#each items as item}
								<div class="group flex items-center gap-3 px-3 py-2 rounded hover:bg-neutral-800 transition-colors">
									<Clock size={14} class="text-neutral-600 shrink-0" />
									<button
										onclick={() => navigateTo(item.url)}
										class="flex-1 min-w-0 text-left"
									>
										<div class="text-sm text-neutral-200 truncate">
											{item.title || item.url}
										</div>
										<div class="text-xs text-neutral-500 truncate">
											{item.url}
										</div>
									</button>
									<span class="text-xxs text-neutral-600 shrink-0">
										{new Date(item.last_visited + 'Z').toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })}
									</span>
									<button
										onclick={() => handleDelete(item.id)}
										class="opacity-0 group-hover:opacity-100 text-neutral-500 hover:text-red-400 transition-opacity shrink-0"
										title="Remove"
									>
										<X size={14} />
									</button>
								</div>
							{/each}
						</div>
					</div>
				{/each}
			{/if}
		</div>
	</div>
{/if}
