<script>
	import { clickOutside } from '$lib/actions/clickOutside.js'

	let { x = 0, y = 0, items = [], visible = false, onClose } = $props()
</script>

{#if visible}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		use:clickOutside={onClose}
		class="fixed bg-neutral-800 border border-neutral-700 rounded-md shadow-lg py-1 z-50 min-w-[180px]"
		style="left: {x}px; top: {y}px;"
	>
		{#each items as item}
			{#if item.separator}
				<div class="my-1 border-t border-neutral-700"></div>
			{:else}
				<button
					onclick={() => {
						item.action?.()
						onClose?.()
					}}
					disabled={item.disabled}
					class="w-full px-3 py-1.5 text-left text-sm text-neutral-300 hover:bg-neutral-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
				>
					{item.label}
				</button>
			{/if}
		{/each}
	</div>
{/if}
