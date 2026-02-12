import { writable } from 'svelte/store'
import { invoke } from '@tauri-apps/api/core'

function createSettingsStore() {
	const { subscribe, set, update } = writable({})

	return {
		subscribe,

		async load() {
			try {
				const all = await invoke('settings_get_all')
				set(all)
			} catch (e) {
				console.error('Failed to load settings:', e)
			}
		},

		async get(key) {
			try {
				return await invoke('settings_get', { key })
			} catch (e) {
				console.error('Failed to get setting:', e)
				return null
			}
		},

		async set(key, value) {
			try {
				await invoke('settings_set', { key, value })
				update((state) => ({ ...state, [key]: value }))
			} catch (e) {
				console.error('Failed to set setting:', e)
			}
		},
	}
}

export const settings = createSettingsStore()
