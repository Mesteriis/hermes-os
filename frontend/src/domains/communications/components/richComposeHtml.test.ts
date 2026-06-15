import { describe, expect, it } from 'vitest'
import {
	appendHtmlSignature,
	appendPlainTextSignature,
	htmlToComposePlainText,
	normalizeMailComposeLinkHref,
	normalizeMailComposeTextAlign,
	plainTextToComposeHtml,
	sanitizeMailComposePastedHtml
} from './richComposeHtml'

describe('rich compose HTML helpers', () => {
	it('escapes plain text and preserves paragraph breaks for rich compose mode', () => {
		expect(plainTextToComposeHtml('Hello <team>\n\nLine & two')).toBe(
			'<p>Hello &lt;team&gt;</p><p>Line &amp; two</p>'
		)
	})

	it('returns an empty paragraph for blank input so the editor remains focusable', () => {
		expect(plainTextToComposeHtml('  \n ')).toBe('<p></p>')
	})

	it('derives a plain text fallback from rich compose HTML', () => {
		expect(htmlToComposePlainText('<p>Hello <strong>team</strong></p><ul><li>One</li></ul>')).toBe(
			'Hello team\nOne'
		)
	})

	it('appends plain and HTML signatures without dropping existing body text', () => {
		expect(appendPlainTextSignature('Hello', 'Alex')).toBe('Hello\n\nAlex')
		expect(appendHtmlSignature('<p>Hello</p>', 'Alex <Lead>')).toBe(
			'<p>Hello</p><p></p><p>Alex &lt;Lead&gt;</p>'
		)
	})

	it('normalizes safe compose link hrefs and rejects active content schemes', () => {
		expect(normalizeMailComposeLinkHref('example.com/path')).toBe('https://example.com/path')
		expect(normalizeMailComposeLinkHref('https://example.com/a?b=1')).toBe(
			'https://example.com/a?b=1'
		)
		expect(normalizeMailComposeLinkHref('mailto:team@example.com')).toBe('mailto:team@example.com')
		expect(normalizeMailComposeLinkHref('javascript:alert(1)')).toBeNull()
		expect(normalizeMailComposeLinkHref('data:text/html,<script>alert(1)</script>')).toBeNull()
		expect(normalizeMailComposeLinkHref('  ')).toBeNull()
		expect(normalizeMailComposeLinkHref('https://')).toBeNull()
		expect(normalizeMailComposeLinkHref('mailto:')).toBeNull()
	})

	it('normalizes only supported mail compose text alignment values', () => {
		expect(normalizeMailComposeTextAlign(' center ')).toBe('center')
		expect(normalizeMailComposeTextAlign('RIGHT')).toBe('right')
		expect(normalizeMailComposeTextAlign('justify')).toBeNull()
		expect(normalizeMailComposeTextAlign('position:absolute')).toBeNull()
	})

	it('sanitizes pasted rich HTML to the supported mail-safe compose subset', () => {
		expect(
			sanitizeMailComposePastedHtml(
				'<p style="color:red"><strong onclick="x()">Hi</strong> <a href="javascript:alert(1)">bad</a> <a href="https://example.com/path" onclick="x()">ok</a><img src=x onerror=x></p><script>alert(1)</script>'
			)
		).toBe(
			'<p><strong>Hi</strong> bad <a href="https://example.com/path" rel="noopener noreferrer" target="_blank">ok</a></p>'
		)
	})

	it('preserves pasted office-style ordered lists in the supported rich compose schema', () => {
		expect(sanitizeMailComposePastedHtml('<ol><li>One</li><li><em>Two</em></li></ol>')).toBe(
			'<ol><li>One</li><li><em>Two</em></li></ol>'
		)
	})

	it('preserves pasted block quotes without unsafe attributes', () => {
		expect(
			sanitizeMailComposePastedHtml('<blockquote cite="https://example.com"><p>Quoted <b>text</b></p></blockquote>')
		).toBe(
			'<blockquote><p>Quoted <strong>text</strong></p></blockquote>'
		)
	})

	it('preserves mail-safe pasted headings and normalizes oversized headings', () => {
		expect(sanitizeMailComposePastedHtml('<h1 onclick="x()">Title</h1><h3>Section</h3>')).toBe(
			'<h2>Title</h2><h3>Section</h3>'
		)
	})

	it('preserves only safe pasted text alignment styles on supported block nodes', () => {
		expect(
			sanitizeMailComposePastedHtml(
				'<p style="text-align:center;color:red">Centered</p><h2 style="text-align: right">Title</h2><p style="text-align:justify">Wide</p>'
			)
		).toBe(
			'<p style="text-align: center">Centered</p><h2 style="text-align: right">Title</h2><p>Wide</p>'
		)
	})
})
