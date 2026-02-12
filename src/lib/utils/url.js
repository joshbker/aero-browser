/**
 * Check if a string looks like a URL (vs a search query)
 */
export const isValidUrl = (input) => {
	// Internal aero:// URLs
	if (/^aero:\/\//i.test(input)) return true

	// Has a protocol
	if (/^https?:\/\//i.test(input)) return true

	// Looks like a domain (has a dot, no spaces)
	if (/^[^\s]+\.[^\s]+$/.test(input) && !input.includes(' ')) return true

	// localhost with optional port
	if (/^localhost(:\d+)?/.test(input)) return true

	return false
}

/**
 * Check if a URL is an internal aero:// page
 */
export const isAeroUrl = (url) => {
	return /^aero:\/\//i.test(url)
}

/**
 * Normalise user input into a navigable URL
 * - If it looks like a URL, prepend https:// if needed
 * - Otherwise, turn it into a Google search
 */
export const resolveInput = (input) => {
	const trimmed = input.trim()
	if (!trimmed) return null

	// Preserve aero:// URLs as-is
	if (/^aero:\/\//i.test(trimmed)) {
		return trimmed
	}

	if (/^https?:\/\//i.test(trimmed)) {
		return trimmed
	}

	if (isValidUrl(trimmed)) {
		return `https://${trimmed}`
	}

	// It's a search query
	return `https://www.google.com/search?q=${encodeURIComponent(trimmed)}`
}

/**
 * Extract a display-friendly version of a URL
 */
export const displayUrl = (url) => {
	// Internal aero:// URLs display as-is
	if (isAeroUrl(url)) return url

	try {
		const parsed = new URL(url)
		// Remove protocol and trailing slash
		let display = parsed.host + parsed.pathname
		if (display.endsWith('/')) display = display.slice(0, -1)
		return display
	} catch {
		return url
	}
}
