import { writable, derived } from 'svelte/store'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

function createTabStore() {
	const { subscribe, set, update } = writable({
		tabs: [],
		activeTabLabel: null,
	})

	// Listen for backend events
	let listenersSetUp = false

	async function setupListeners() {
		if (listenersSetUp) return
		listenersSetUp = true

		await listen('tab_created', (event) => {
			const tab = event.payload
			update((state) => {
				// Avoid duplicates — backend may emit before invoke returns
				const exists = state.tabs.some((t) => t.label === tab.label)
				if (exists) return state
				return {
					...state,
					tabs: [...state.tabs, tab],
					activeTabLabel: tab.label,
				}
			})
		})

		await listen('tab_closed', (event) => {
			const { label } = event.payload
			update((state) => ({
				...state,
				tabs: state.tabs.filter((t) => t.label !== label),
			}))
		})

		await listen('tab_updated', (event) => {
			const { label, loading, url, title, favicon } = event.payload
			update((state) => ({
				...state,
				tabs: state.tabs.map((tab) =>
					tab.label === label
						? {
								...tab,
								...(loading !== undefined && { is_loading: loading }),
								...(url !== undefined && { url }),
								...(title !== undefined && { title }),
								...(favicon !== undefined && { favicon }),
							}
						: tab
				),
			}))
		})

		await listen('tab_activated', (event) => {
			const tab = event.payload
			update((state) => ({
				...state,
				activeTabLabel: tab.label,
			}))
		})
	}

	return {
		subscribe,

		async init() {
			await setupListeners()
			// Load existing tabs from backend
			try {
				const tabs = await invoke('tab_get_all')
				const activeLabel = await invoke('tab_get_active')
				set({ tabs, activeTabLabel: activeLabel })
			} catch (e) {
				console.error('Failed to load tabs:', e)
			}

			// If no tabs exist, create the first one
			const state = await new Promise((resolve) => {
				const unsub = subscribe((s) => {
					resolve(s)
					// Unsubscribe on next tick
					setTimeout(() => unsub(), 0)
				})
			})

			if (state.tabs.length === 0) {
				await this.create()
			}
		},

		async create(url) {
			try {
				const tab = await invoke('tab_create', { url: url || null })
				// State will be updated by the tab_created event listener,
				// but we also update optimistically here for responsiveness
				update((state) => {
					const exists = state.tabs.some((t) => t.label === tab.label)
					if (exists) return { ...state, activeTabLabel: tab.label }
					return {
						tabs: [...state.tabs, tab],
						activeTabLabel: tab.label,
					}
				})
				return tab
			} catch (e) {
				console.error('Failed to create tab:', e)
			}
		},

		async close(label) {
			try {
				await invoke('tab_close', { label })
			} catch (e) {
				console.error('Failed to close tab:', e)
			}
		},

		async setActive(label) {
			try {
				// Optimistic update
				update((state) => ({ ...state, activeTabLabel: label }))
				await invoke('tab_set_active', { label })
			} catch (e) {
				console.error('Failed to set active tab:', e)
			}
		},
	}
}

export const tabs = createTabStore()

// Derived store for the active tab object
export const activeTab = derived(tabs, ($tabs) => {
	return $tabs.tabs.find((t) => t.label === $tabs.activeTabLabel) || null
})
