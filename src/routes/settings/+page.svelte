<script>
	import { onMount } from 'svelte'
	import { settings } from '$lib/stores/settings.js'
	import { ChevronRight, Search, Palette, Shield, Globe, Settings } from 'lucide-svelte'

	let loaded = $state(false)
	let activeSection = $state('general')

	onMount(async () => {
		await settings.load()
		loaded = true
	})

	const sections = [
		{ id: 'general', label: 'General', icon: Settings },
		{ id: 'search', label: 'Search', icon: Search },
		{ id: 'appearance', label: 'Appearance', icon: Palette },
		{ id: 'privacy', label: 'Privacy', icon: Shield },
	]

	async function handleChange(key, value) {
		await settings.set(key, value)
	}
</script>

{#if !loaded}
	<div class="flex items-center justify-center h-full text-neutral-500">
		Loading settings...
	</div>
{:else}
	<div class="flex h-full bg-neutral-900 text-neutral-200">
		<!-- Sidebar -->
		<nav class="w-56 shrink-0 border-r border-neutral-700 p-4">
			<h1 class="text-lg font-semibold mb-4 text-neutral-100">Settings</h1>
			<ul class="space-y-1">
				{#each sections as section}
					<li>
						<button
							onclick={() => activeSection = section.id}
							class="flex items-center gap-2 w-full px-3 py-2 rounded text-sm transition-colors
								{activeSection === section.id
									? 'bg-neutral-700 text-neutral-100'
									: 'text-neutral-400 hover:bg-neutral-800 hover:text-neutral-200'}"
						>
							<section.icon size={16} />
							{section.label}
						</button>
					</li>
				{/each}
			</ul>
		</nav>

		<!-- Content -->
		<div class="flex-1 overflow-y-auto p-8 max-w-2xl">
			{#if activeSection === 'general'}
				<h2 class="text-base font-semibold mb-6">General</h2>

				<div class="space-y-6">
					<div>
						<label for="homepage" class="block text-sm text-neutral-400 mb-1">Homepage</label>
						<input
							id="homepage"
							type="text"
							value={$settings.homepage || ''}
							onchange={(e) => handleChange('homepage', e.target.value)}
							class="w-full px-3 py-2 bg-neutral-800 border border-neutral-700 rounded text-sm text-neutral-200 outline-none focus:border-blue-500"
						/>
					</div>

					<div>
						<label for="new_tab_page" class="block text-sm text-neutral-400 mb-1">New tab page</label>
						<input
							id="new_tab_page"
							type="text"
							value={$settings.new_tab_page || ''}
							onchange={(e) => handleChange('new_tab_page', e.target.value)}
							class="w-full px-3 py-2 bg-neutral-800 border border-neutral-700 rounded text-sm text-neutral-200 outline-none focus:border-blue-500"
						/>
					</div>

					<div>
						<label for="restore_on_startup" class="block text-sm text-neutral-400 mb-1">On startup</label>
						<select
							id="restore_on_startup"
							value={$settings.restore_on_startup || 'new_tab'}
							onchange={(e) => handleChange('restore_on_startup', e.target.value)}
							class="w-full px-3 py-2 bg-neutral-800 border border-neutral-700 rounded text-sm text-neutral-200 outline-none focus:border-blue-500"
						>
							<option value="new_tab">Open a new tab</option>
							<option value="restore">Continue where you left off</option>
							<option value="homepage">Open homepage</option>
						</select>
					</div>

					<div>
						<label for="download_path" class="block text-sm text-neutral-400 mb-1">Download location</label>
						<input
							id="download_path"
							type="text"
							value={$settings.download_path || '~/Downloads'}
							onchange={(e) => handleChange('download_path', e.target.value)}
							class="w-full px-3 py-2 bg-neutral-800 border border-neutral-700 rounded text-sm text-neutral-200 outline-none focus:border-blue-500"
						/>
					</div>

					<div class="flex items-center justify-between">
						<label for="ask_download_location" class="text-sm text-neutral-400">Ask where to save each file</label>
						<button
							id="ask_download_location"
							onclick={() => handleChange('ask_download_location', $settings.ask_download_location === 'true' ? 'false' : 'true')}
							class="w-10 h-5 rounded-full transition-colors relative
								{$settings.ask_download_location === 'true' ? 'bg-blue-600' : 'bg-neutral-600'}"
						>
							<span class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white transition-transform
								{$settings.ask_download_location === 'true' ? 'translate-x-5' : 'translate-x-0'}"></span>
						</button>
					</div>
				</div>

			{:else if activeSection === 'search'}
				<h2 class="text-base font-semibold mb-6">Search</h2>

				<div class="space-y-6">
					<div>
						<label for="search_engine" class="block text-sm text-neutral-400 mb-1">Search engine</label>
						<select
							id="search_engine"
							value={$settings.search_engine || 'https://www.google.com/search?q='}
							onchange={(e) => handleChange('search_engine', e.target.value)}
							class="w-full px-3 py-2 bg-neutral-800 border border-neutral-700 rounded text-sm text-neutral-200 outline-none focus:border-blue-500"
						>
							<option value="https://www.google.com/search?q=">Google</option>
							<option value="https://duckduckgo.com/?q=">DuckDuckGo</option>
							<option value="https://www.bing.com/search?q=">Bing</option>
							<option value="https://search.brave.com/search?q=">Brave Search</option>
						</select>
					</div>
				</div>

			{:else if activeSection === 'appearance'}
				<h2 class="text-base font-semibold mb-6">Appearance</h2>

				<div class="space-y-6">
					<div>
						<label for="theme" class="block text-sm text-neutral-400 mb-1">Theme</label>
						<select
							id="theme"
							value={$settings.theme || 'dark'}
							onchange={(e) => handleChange('theme', e.target.value)}
							class="w-full px-3 py-2 bg-neutral-800 border border-neutral-700 rounded text-sm text-neutral-200 outline-none focus:border-blue-500"
						>
							<option value="dark">Dark</option>
							<option value="light">Light</option>
							<option value="system">System</option>
						</select>
					</div>

					<div>
						<label for="default_zoom" class="block text-sm text-neutral-400 mb-1">Default zoom ({$settings.default_zoom || 100}%)</label>
						<input
							id="default_zoom"
							type="range"
							min="50"
							max="200"
							step="10"
							value={$settings.default_zoom || '100'}
							oninput={(e) => handleChange('default_zoom', e.target.value)}
							class="w-full accent-blue-500"
						/>
					</div>

					<div class="flex items-center justify-between">
						<label for="show_bookmarks_bar" class="text-sm text-neutral-400">Show bookmarks bar</label>
						<button
							id="show_bookmarks_bar"
							onclick={() => handleChange('show_bookmarks_bar', $settings.show_bookmarks_bar === 'true' ? 'false' : 'true')}
							class="w-10 h-5 rounded-full transition-colors relative
								{$settings.show_bookmarks_bar === 'true' ? 'bg-blue-600' : 'bg-neutral-600'}"
						>
							<span class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white transition-transform
								{$settings.show_bookmarks_bar === 'true' ? 'translate-x-5' : 'translate-x-0'}"></span>
						</button>
					</div>

					<div class="flex items-center justify-between">
						<label for="show_status_bar" class="text-sm text-neutral-400">Show status bar</label>
						<button
							id="show_status_bar"
							onclick={() => handleChange('show_status_bar', $settings.show_status_bar === 'true' ? 'false' : 'true')}
							class="w-10 h-5 rounded-full transition-colors relative
								{$settings.show_status_bar === 'true' ? 'bg-blue-600' : 'bg-neutral-600'}"
						>
							<span class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white transition-transform
								{$settings.show_status_bar === 'true' ? 'translate-x-5' : 'translate-x-0'}"></span>
						</button>
					</div>
				</div>

			{:else if activeSection === 'privacy'}
				<h2 class="text-base font-semibold mb-6">Privacy & Security</h2>

				<div class="space-y-4">
					<p class="text-sm text-neutral-500">Privacy settings will be available in a future update.</p>
				</div>
			{/if}
		</div>
	</div>
{/if}
