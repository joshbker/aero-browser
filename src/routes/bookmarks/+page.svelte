<script>
	import { onMount } from 'svelte'
	import { invoke } from '@tauri-apps/api/core'
	import { bookmarks, BOOKMARKS_BAR } from '$lib/stores/bookmarks.js'
	import { Search, Folder, Star, Trash2, Edit2, ChevronRight, ChevronDown } from 'lucide-svelte'

	let allBookmarks = $state([])
	let searchQuery = $state('')
	let searchResults = $state([])
	let loaded = $state(false)
	let expandedFolders = $state(new Set(['bookmarks-bar', 'other-bookmarks']))
	let editingId = $state(null)
	let editTitle = $state('')
	let editUrl = $state('')

	onMount(async () => {
		await loadAll()
		loaded = true
	})

	async function loadAll() {
		try {
			allBookmarks = await invoke('bookmark_get_all')
		} catch (e) {
			console.error('Failed to load bookmarks:', e)
		}
	}

	async function handleSearch() {
		if (searchQuery.trim()) {
			try {
				searchResults = await invoke('bookmark_search', { query: searchQuery, limit: 50 })
			} catch {
				searchResults = []
			}
		} else {
			searchResults = []
		}
	}

	function getChildren(parentId) {
		return allBookmarks.filter((b) => b.parent_id === parentId).sort((a, b) => a.position - b.position)
	}

	function getRootFolders() {
		return allBookmarks.filter((b) => b.parent_id === null && b.is_folder)
	}

	function toggleFolder(id) {
		const next = new Set(expandedFolders)
		if (next.has(id)) {
			next.delete(id)
		} else {
			next.add(id)
		}
		expandedFolders = next
	}

	function navigateTo(url) {
		if (url) invoke('navigate_to', { url })
	}

	async function deleteBookmark(id) {
		await bookmarks.remove(id)
		await loadAll()
	}

	function startEdit(bm) {
		editingId = bm.id
		editTitle = bm.title
		editUrl = bm.url || ''
	}

	async function saveEdit() {
		if (editingId) {
			await bookmarks.update(editingId, editTitle, editUrl || null)
			editingId = null
			await loadAll()
		}
	}

	function cancelEdit() {
		editingId = null
	}
</script>

{#if !loaded}
	<div class="flex items-center justify-center h-full text-neutral-500">
		Loading bookmarks...
	</div>
{:else}
	<div class="flex flex-col h-full bg-neutral-900 text-neutral-200">
		<div class="p-6 pb-4">
			<h1 class="text-lg font-semibold mb-4">Bookmarks</h1>

			<!-- Search -->
			<div class="flex items-center gap-2 h-8 px-3 bg-neutral-800 border border-neutral-700 rounded focus-within:border-blue-500">
				<Search size={14} class="text-neutral-500 shrink-0" />
				<input
					bind:value={searchQuery}
					oninput={handleSearch}
					placeholder="Search bookmarks"
					class="flex-1 bg-transparent text-sm text-neutral-200 placeholder-neutral-500 outline-none"
				/>
			</div>
		</div>

		<div class="flex-1 overflow-y-auto px-6">
			{#if searchQuery.trim() && searchResults.length > 0}
				<!-- Search results -->
				<div class="space-y-0.5">
					{#each searchResults as bm}
						<div class="group flex items-center gap-3 px-3 py-2 rounded hover:bg-neutral-800">
							<Star size={14} class="text-yellow-400 shrink-0" />
							<button onclick={() => navigateTo(bm.url)} class="flex-1 min-w-0 text-left">
								<div class="text-sm text-neutral-200 truncate">{bm.title}</div>
								<div class="text-xs text-neutral-500 truncate">{bm.url}</div>
							</button>
							<button onclick={() => deleteBookmark(bm.id)} class="opacity-0 group-hover:opacity-100 text-neutral-500 hover:text-red-400">
								<Trash2 size={14} />
							</button>
						</div>
					{/each}
				</div>
			{:else if searchQuery.trim()}
				<p class="text-neutral-500 text-sm">No bookmarks found.</p>
			{:else}
				<!-- Tree view -->
				{#each getRootFolders() as folder}
					{@const isExpanded = expandedFolders.has(folder.id)}
					{@const children = getChildren(folder.id)}
					<div class="mb-2">
						<button
							onclick={() => toggleFolder(folder.id)}
							class="flex items-center gap-2 w-full px-3 py-2 text-sm font-medium text-neutral-300 hover:bg-neutral-800 rounded"
						>
							{#if isExpanded}
								<ChevronDown size={14} />
							{:else}
								<ChevronRight size={14} />
							{/if}
							<Folder size={14} />
							{folder.title}
							<span class="text-xs text-neutral-600 ml-1">({children.length})</span>
						</button>

						{#if isExpanded}
							<div class="ml-6 space-y-0.5">
								{#each children as child}
									{#if child.is_folder}
										{@const subChildren = getChildren(child.id)}
										{@const subExpanded = expandedFolders.has(child.id)}
										<div>
											<button
												onclick={() => toggleFolder(child.id)}
												class="flex items-center gap-2 w-full px-3 py-1.5 text-sm text-neutral-400 hover:bg-neutral-800 rounded"
											>
												{#if subExpanded}
													<ChevronDown size={12} />
												{:else}
													<ChevronRight size={12} />
												{/if}
												<Folder size={12} />
												{child.title}
												<span class="text-xs text-neutral-600 ml-1">({subChildren.length})</span>
											</button>
											{#if subExpanded}
												<div class="ml-6 space-y-0.5">
													{#each subChildren as sub}
														<div class="group flex items-center gap-2 px-3 py-1.5 rounded hover:bg-neutral-800">
															{#if editingId === sub.id}
																<input bind:value={editTitle} class="flex-1 text-sm bg-neutral-700 px-2 py-0.5 rounded text-neutral-200 outline-none" />
																<input bind:value={editUrl} class="flex-1 text-xs bg-neutral-700 px-2 py-0.5 rounded text-neutral-400 outline-none" />
																<button onclick={saveEdit} class="text-xs text-blue-400">Save</button>
																<button onclick={cancelEdit} class="text-xs text-neutral-500">Cancel</button>
															{:else}
																<Star size={12} class="text-yellow-400 shrink-0" />
																<button onclick={() => navigateTo(sub.url)} class="flex-1 min-w-0 text-left">
																	<span class="text-sm text-neutral-300 truncate block">{sub.title}</span>
																</button>
																<button onclick={() => startEdit(sub)} class="opacity-0 group-hover:opacity-100 text-neutral-500 hover:text-neutral-300">
																	<Edit2 size={12} />
																</button>
																<button onclick={() => deleteBookmark(sub.id)} class="opacity-0 group-hover:opacity-100 text-neutral-500 hover:text-red-400">
																	<Trash2 size={12} />
																</button>
															{/if}
														</div>
													{/each}
												</div>
											{/if}
										</div>
									{:else}
										<div class="group flex items-center gap-2 px-3 py-1.5 rounded hover:bg-neutral-800">
											{#if editingId === child.id}
												<input bind:value={editTitle} class="flex-1 text-sm bg-neutral-700 px-2 py-0.5 rounded text-neutral-200 outline-none" />
												<input bind:value={editUrl} class="flex-1 text-xs bg-neutral-700 px-2 py-0.5 rounded text-neutral-400 outline-none" />
												<button onclick={saveEdit} class="text-xs text-blue-400">Save</button>
												<button onclick={cancelEdit} class="text-xs text-neutral-500">Cancel</button>
											{:else}
												<Star size={12} class="text-yellow-400 shrink-0" />
												<button onclick={() => navigateTo(child.url)} class="flex-1 min-w-0 text-left">
													<span class="text-sm text-neutral-300 truncate block">{child.title}</span>
													<span class="text-xs text-neutral-500 truncate block">{child.url}</span>
												</button>
												<button onclick={() => startEdit(child)} class="opacity-0 group-hover:opacity-100 text-neutral-500 hover:text-neutral-300">
													<Edit2 size={12} />
												</button>
												<button onclick={() => deleteBookmark(child.id)} class="opacity-0 group-hover:opacity-100 text-neutral-500 hover:text-red-400">
													<Trash2 size={12} />
												</button>
											{/if}
										</div>
									{/if}
								{/each}
							</div>
						{/if}
					</div>
				{/each}
			{/if}
		</div>
	</div>
{/if}
