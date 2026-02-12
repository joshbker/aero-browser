export function clickOutside(node, callback) {
	function handleClick(e) {
		if (!node.contains(e.target)) {
			callback?.()
		}
	}

	document.addEventListener('mousedown', handleClick, true)

	return {
		destroy() {
			document.removeEventListener('mousedown', handleClick, true)
		}
	}
}
