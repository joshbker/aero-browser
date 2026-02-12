import { writable } from 'svelte/store'
import { invoke } from '@tauri-apps/api/core'

function createHistoryStore() {
	const { subscribe, set, update } = writable([])

	return {
		subscribe,

		async search(query) {
			try {
				const results = await invoke('history_search', { query, limit: 50 })
				set(results)
				return results
			} catch (e) {
				console.error('Failed to search history:', e)
				return []
			}
		},

		async getRecent(limit = 100) {
			try {
				const results = await invoke('history_get_recent', { limit })
				set(results)
				return results
			} catch (e) {
				console.error('Failed to get recent history:', e)
				return []
			}
		},

		async delete(id) {
			try {
				await invoke('history_delete', { id })
				update((items) => items.filter((i) => i.id !== id))
			} catch (e) {
				console.error('Failed to delete history entry:', e)
			}
		},

		async clear(timeframe) {
			try {
				await invoke('history_clear', { timeframe })
				if (timeframe === 'all') {
					set([])
				} else {
					// Reload to reflect cleared items
					const results = await invoke('history_get_recent', { limit: 100 })
					set(results)
				}
			} catch (e) {
				console.error('Failed to clear history:', e)
			}
		},

		async suggest(query, limit = 8) {
			try {
				return await invoke('history_search', { query, limit })
			} catch (e) {
				return []
			}
		},
	}
}

export const history = createHistoryStore()
