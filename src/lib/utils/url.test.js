import { describe, test, expect } from 'vitest'
import { isValidUrl, isAeroUrl, resolveInput, displayUrl } from './url.js'

// ── isValidUrl ─────────────────────────────────────────

describe('isValidUrl', () => {
	test('accepts http:// URLs', () => {
		expect(isValidUrl('http://example.com')).toBe(true)
	})

	test('accepts https:// URLs', () => {
		expect(isValidUrl('https://google.com')).toBe(true)
	})

	test('accepts aero:// URLs', () => {
		expect(isValidUrl('aero://settings')).toBe(true)
		expect(isValidUrl('aero://history')).toBe(true)
		expect(isValidUrl('aero://bookmarks')).toBe(true)
	})

	test('accepts bare domains with TLD', () => {
		expect(isValidUrl('google.com')).toBe(true)
		expect(isValidUrl('example.co.uk')).toBe(true)
		expect(isValidUrl('sub.domain.org')).toBe(true)
	})

	test('accepts localhost', () => {
		expect(isValidUrl('localhost')).toBe(true)
		expect(isValidUrl('localhost:3000')).toBe(true)
		expect(isValidUrl('localhost:8080')).toBe(true)
	})

	test('rejects plain words', () => {
		expect(isValidUrl('hello')).toBe(false)
		expect(isValidUrl('search query')).toBe(false)
	})

	test('rejects phrases with spaces', () => {
		expect(isValidUrl('how to cook pasta')).toBe(false)
		expect(isValidUrl('what is javascript')).toBe(false)
	})
})

// ── isAeroUrl ─────────────────────────────────────────

describe('isAeroUrl', () => {
	test('detects aero:// URLs', () => {
		expect(isAeroUrl('aero://settings')).toBe(true)
		expect(isAeroUrl('aero://history')).toBe(true)
	})

	test('rejects non-aero URLs', () => {
		expect(isAeroUrl('https://example.com')).toBe(false)
		expect(isAeroUrl('http://settings')).toBe(false)
		expect(isAeroUrl('settings')).toBe(false)
	})
})

// ── resolveInput ───────────────────────────────────────

describe('resolveInput', () => {
	test('preserves full http:// URLs', () => {
		expect(resolveInput('http://example.com')).toBe('http://example.com')
	})

	test('preserves full https:// URLs', () => {
		expect(resolveInput('https://github.com/foo')).toBe('https://github.com/foo')
	})

	test('preserves aero:// URLs', () => {
		expect(resolveInput('aero://settings')).toBe('aero://settings')
		expect(resolveInput('aero://history')).toBe('aero://history')
	})

	test('prepends https:// to bare domains', () => {
		expect(resolveInput('google.com')).toBe('https://google.com')
		expect(resolveInput('example.co.uk')).toBe('https://example.co.uk')
	})

	test('prepends https:// to localhost', () => {
		expect(resolveInput('localhost:3000')).toBe('https://localhost:3000')
	})

	test('converts search queries to Google search', () => {
		const result = resolveInput('how to cook pasta')
		expect(result).toContain('google.com/search?q=')
		expect(result).toContain('how%20to%20cook%20pasta')
	})

	test('trims whitespace', () => {
		expect(resolveInput('  google.com  ')).toBe('https://google.com')
	})

	test('returns null for empty input', () => {
		expect(resolveInput('')).toBeNull()
		expect(resolveInput('   ')).toBeNull()
	})

	test('encodes special characters in search', () => {
		const result = resolveInput('c++ tutorial')
		expect(result).toContain('google.com/search?q=')
		expect(result).toContain('c%2B%2B')
	})
})

// ── displayUrl ─────────────────────────────────────────

describe('displayUrl', () => {
	test('strips protocol and trailing slash', () => {
		expect(displayUrl('https://google.com/')).toBe('google.com')
	})

	test('preserves path', () => {
		expect(displayUrl('https://example.com/about')).toBe('example.com/about')
	})

	test('preserves subdomain', () => {
		expect(displayUrl('https://www.github.com/')).toBe('www.github.com')
	})

	test('returns input for invalid URL', () => {
		expect(displayUrl('not a url')).toBe('not a url')
	})

	test('handles URL with port', () => {
		expect(displayUrl('http://localhost:3000/app')).toBe('localhost:3000/app')
	})

	test('displays aero:// URLs as-is', () => {
		expect(displayUrl('aero://settings')).toBe('aero://settings')
		expect(displayUrl('aero://history')).toBe('aero://history')
	})
})
