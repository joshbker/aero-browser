import { defineConfig } from 'vite'
import { sveltekit } from '@sveltejs/kit/vite'

const host = process.env.TAURI_DEV_HOST

export default defineConfig({
	plugins: [sveltekit()],
	envPrefix: ['VITE_', 'TAURI_'],
	build: {
		target: ['es2021', 'chrome97'],
		minify: !process.env.TAURI_DEBUG && 'esbuild',
		sourcemap: !!process.env.TAURI_DEBUG,
	},
	clearScreen: false,
	server: {
		port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host
			? {
					protocol: 'ws',
					host,
					port: 1421,
				}
			: undefined,
		watch: {
			ignored: ['**/src-tauri/**'],
		},
	},
})
