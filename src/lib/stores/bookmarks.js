import { writable, derived } from 'svelte/store'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const BOOKMARKS_BAR_ID = 'bookmarks-bar'

function createBookmarksStore() {
	const { subscribe, set, update } = writable({
		barItems: [],
		showBar: true,
	})

	let listenersSetUp = false

	async function setupListeners() {
		if (listenersSetUp) return
		listenersSetUp = true

		await listen('chrome_height_changed', () => {
			// Chrome height changed — UI will re-render
		})
	}

	return {
		subscribe,

		async init() {
			await setupListeners()
			await this.loadBar()
			// Load show_bookmarks_bar setting
			try {
				const val = await invoke('settings_get', { key: 'show_bookmarks_bar' })
				const visible = val !== 'false'
				update((s) => ({ ...s, showBar: visible }))
			} catch {
				// Default to true
			}
		},

		async loadBar() {
			try {
				const items = await invoke('bookmark_get_children', { parentId: BOOKMARKS_BAR_ID })
				update((s) => ({ ...s, barItems: items }))
			} catch (e) {
				console.error('Failed to load bookmarks bar:', e)
			}
		},

		async add(parentId, title, url, isFolder = false) {
			try {
				const bm = await invoke('bookmark_add', {
					parentId: parentId || BOOKMARKS_BAR_ID,
					title,
					url: url || null,
					isFolder,
				})
				await this.loadBar()
				return bm
			} catch (e) {
				console.error('Failed to add bookmark:', e)
			}
		},

		async remove(id) {
			try {
				await invoke('bookmark_delete', { id })
				await this.loadBar()
			} catch (e) {
				console.error('Failed to delete bookmark:', e)
			}
		},

		async update(id, title, url) {
			try {
				await invoke('bookmark_update', {
					id,
					title: title || null,
					url: url || null,
				})
				await this.loadBar()
			} catch (e) {
				console.error('Failed to update bookmark:', e)
			}
		},

		async isBookmarked(url) {
			try {
				return await invoke('bookmark_is_bookmarked', { url })
			} catch {
				return null
			}
		},

		async toggleBookmark(url, title) {
			const existingId = await this.isBookmarked(url)
			if (existingId) {
				await this.remove(existingId)
				return null
			} else {
				return await this.add(BOOKMARKS_BAR_ID, title || url, url)
			}
		},

		async toggleBar() {
			let newState
			update((s) => {
				newState = !s.showBar
				return { ...s, showBar: newState }
			})
			try {
				await invoke('bookmark_toggle_bar', { visible: newState })
			} catch (e) {
				console.error('Failed to toggle bookmarks bar:', e)
			}
		},

		async getChildren(parentId) {
			try {
				return await invoke('bookmark_get_children', { parentId })
			} catch (e) {
				console.error('Failed to get children:', e)
				return []
			}
		},
	}
}

export const bookmarks = createBookmarksStore()
export const BOOKMARKS_BAR = BOOKMARKS_BAR_ID
