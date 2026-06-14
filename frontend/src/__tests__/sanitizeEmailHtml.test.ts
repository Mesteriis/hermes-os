import { describe, expect, it } from 'vitest'
import { renderMessageBody, sanitizeEmailHtml } from '@/shared/sanitize/emailHtml'

describe('sanitizeEmailHtml', () => {
	it('removes active content, event handlers, inline styles, and javascript urls', () => {
		const sanitized = sanitizeEmailHtml(`
			<div onclick="steal()" style="color: red">
				<script>alert('x')</script>
				<a href="javascript:alert(1)" onmouseover="steal()">open</a>
				<img src="javascript:alert(1)" onerror="steal()" alt="tracker">
				<form action="/send"><button>Send</button></form>
				<svg><script>alert('svg')</script></svg>
			</div>
		`)

		expect(sanitized).toContain('<div>')
		expect(sanitized).toContain('<a>open</a>')
		expect(sanitized).toContain('<img alt="tracker">')
		expect(sanitized).not.toContain('script')
		expect(sanitized).not.toContain('onclick')
		expect(sanitized).not.toContain('onmouseover')
		expect(sanitized).not.toContain('onerror')
		expect(sanitized).not.toContain('style=')
		expect(sanitized).not.toContain('javascript:')
		expect(sanitized).not.toContain('<form')
		expect(sanitized).not.toContain('<svg')
	})

	it('keeps a constrained set of email formatting tags and safe links', () => {
		const sanitized = sanitizeEmailHtml(`
			<p>Hello <b>Alex</b>, <i>see</i> <a href="https://example.com?q=1&x=2">details</a>.</p>
			<blockquote cite="https://ignored.example">Quoted</blockquote>
			<table><tr><td colspan="2">Cell</td></tr></table>
		`)

		expect(sanitized).toContain('<p>Hello <strong>Alex</strong>, <em>see</em> ')
		expect(sanitized).toContain(
			'<a href="https://example.com?q=1&amp;x=2" target="_blank" rel="noreferrer noopener">details</a>'
		)
		expect(sanitized).toContain('<blockquote>Quoted</blockquote>')
		expect(sanitized).toContain('<table><tr><td colspan="2">Cell</td></tr></table>')
		expect(sanitized).not.toContain('cite=')
	})

	it('rejects obfuscated unsafe urls without throwing on malformed entities', () => {
		const sanitized = sanitizeEmailHtml(`
			<a href="java&#x0a;script&colon;alert(1)">bad link</a>
			<img src="mailto:person@example.com" alt="bad image">
			<a href="&#999999999999999999999999;">invalid entity</a>
		`)

		expect(sanitized).toContain('<a>bad link</a>')
		expect(sanitized).toContain('<img alt="bad image">')
		expect(sanitized).toContain('<a>invalid entity</a>')
		expect(sanitized).not.toContain('javascript')
		expect(sanitized).not.toContain('mailto:person@example.com')
	})
})

describe('renderMessageBody', () => {
	it('renders HTML bodies through the sanitizer', () => {
		const rendered = renderMessageBody({
			bodyHtml: '<p>Safe</p><script>alert(1)</script>',
			bodyText: 'fallback'
		})

		expect(rendered.kind).toBe('html')
		expect(rendered.html).toBe('<p>Safe</p>')
	})

	it('escapes plain text and preserves line breaks', () => {
		const rendered = renderMessageBody({
			bodyHtml: null,
			bodyText: 'Hello <script>alert(1)</script>\nSecond line'
		})

		expect(rendered.kind).toBe('plain')
		expect(rendered.html).toBe('Hello &lt;script&gt;alert(1)&lt;/script&gt;<br>Second line')
	})
})
