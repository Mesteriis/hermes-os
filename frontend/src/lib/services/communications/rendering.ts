import type {
	CommunicationMessageSummary,
	CommunicationMessageSummaryV2
} from '$lib/api';
import type { OriginalMailSrcdocOptions, RenderedMessageContent } from './types';

export function conversationPreview(
	message:
		| CommunicationMessageSummary
		| CommunicationMessageSummaryV2
		| (CommunicationMessageSummary & { ai_summary?: string | null })
): string {
	const aiSummary = 'ai_summary' in message ? cleanPreviewText(message.ai_summary ?? '') : '';
	if (aiSummary) return truncatePreview(aiSummary);

	const bodyPreview = cleanPreviewText(message.body_text_preview);
	if (bodyPreview) return truncatePreview(bodyPreview);

	return truncatePreview(cleanPreviewText(message.subject) || message.subject.trim());
}

export function messageContentText(value: string): string {
	return cleanPreviewText(value);
}

export function renderMessageContent(value: string): RenderedMessageContent {
	const withoutHeaders = stripLeadingMimeHeaderBlock(value);
	const decodedHtml = looksLikeEscapedHtml(withoutHeaders)
		? decodeBasicEntities(withoutHeaders)
		: withoutHeaders;

	if (looksLikeHtml(decodedHtml)) {
		return {
			html: sanitizeEmailHtml(decodedHtml),
			mode: 'html'
		};
	}

	return {
		html: renderPlainMessageText(withoutHeaders),
		mode: 'text'
	};
}

export function originalMailSrcdoc(
	value: string,
	options: OriginalMailSrcdocOptions = {}
): string {
	const html = rewriteRemoteMailImageSources(value.trim(), options);
	if (!html) return '';
	const base = '<base target="_blank">';
	if (/<html[\s>]/i.test(html)) {
		if (/<head[\s>]/i.test(html)) {
			return html.replace(/<head(\s[^>]*)?>/i, (match) => `${match}${base}`);
		}
		return html.replace(/<html(\s[^>]*)?>/i, (match) => `${match}<head>${base}</head>`);
	}
	return `<!doctype html><html><head>${base}</head><body>${html}</body></html>`;
}

export function remoteMailImageProxyUrl(
	messageId: string,
	imageUrl: string,
	apiBaseUrl: string
): string {
	const proxyUrl = new URL(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/remote-image`,
		apiBaseUrl.replace(/\/+$/, '') + '/'
	);
	proxyUrl.searchParams.set('url', decodeBasicEntities(imageUrl.trim()));
	return proxyUrl.toString();
}

function rewriteRemoteMailImageSources(
	html: string,
	options: OriginalMailSrcdocOptions
): string {
	const messageId = options.messageId?.trim();
	const apiBaseUrl = options.apiBaseUrl?.trim();
	if (!messageId || !apiBaseUrl) return html;

	return rewriteRemoteMailCssImageUrls(
		html.replace(/<img\b[^>]*>/gi, (tag) =>
			rewriteRemoteMailImageAttribute(tag, 'src', messageId, apiBaseUrl)
		),
		messageId,
		apiBaseUrl
	).replace(/<(?:table|td|th|div|body)\b[^>]*>/gi, (tag) =>
		rewriteRemoteMailImageAttribute(tag, 'background', messageId, apiBaseUrl)
	);
}

function rewriteRemoteMailImageAttribute(
	tag: string,
	attributeName: 'background' | 'src',
	messageId: string,
	apiBaseUrl: string
): string {
	const attributePattern = new RegExp(
		`(\\s+${attributeName}\\s*=\\s*)(?:"([^"]*)"|'([^']*)'|([^\\s"'=<>` + '`' + `]+))`,
		'i'
	);
	return tag.replace(
		attributePattern,
		(match, prefix: string, doubleQuoted?: string, singleQuoted?: string, unquoted?: string) => {
			const src = doubleQuoted ?? singleQuoted ?? unquoted ?? '';
			if (!isRemoteMailImageSource(src)) return match;
			const proxyUrl = remoteMailImageProxyUrl(messageId, src, apiBaseUrl);
			if (doubleQuoted !== undefined) return `${prefix}"${escapeHtml(proxyUrl)}"`;
			if (singleQuoted !== undefined) return `${prefix}'${escapeHtml(proxyUrl)}'`;
			return `${prefix}${escapeHtml(proxyUrl)}`;
		}
	);
}

function rewriteRemoteMailCssImageUrls(
	html: string,
	messageId: string,
	apiBaseUrl: string
): string {
	return html.replace(
		/url\(\s*(?:"([^"]*)"|'([^']*)'|([^'")\s]+))\s*\)/gi,
		(match, doubleQuoted?: string, singleQuoted?: string, unquoted?: string) => {
			const src = doubleQuoted ?? singleQuoted ?? unquoted ?? '';
			if (!isRemoteMailImageSource(src)) return match;
			const proxyUrl = remoteMailImageProxyUrl(messageId, src, apiBaseUrl);
			return `url('${escapeHtml(proxyUrl)}')`;
		}
	);
}

function isRemoteMailImageSource(value: string): boolean {
	const decoded = decodeBasicEntities(value).trim();
	return /^https?:\/\//i.test(decoded);
}

function cleanPreviewText(value: string): string {
	const withoutBlocks = value
		.replace(/<style\b[^>]*>[\s\S]*?<\/style>/gi, ' ')
		.replace(/<script\b[^>]*>[\s\S]*?<\/script>/gi, ' ')
		.replace(/<\/?(?:div|p|span|br|table|tbody|tr|td|th|html|body|head)[^>]*>/gi, ' ')
		.replace(/<[^>]+>/g, ' ');
	const cleanedLines = withoutBlocks
		.split(/\r?\n/)
		.map((line) => decodeBasicEntities(line).trim())
		.filter((line) => line && !looksLikeMimeHeader(line) && !looksLikeCssDeclaration(line) && !looksLikeCssRule(line));
	return cleanedLines.join(' ').replace(/\s+/g, ' ').trim();
}

function renderPlainMessageText(value: string): string {
	const cleaned = value
		.split(/\r?\n/)
		.filter((line) => !looksLikeMimeHeader(line) && !looksLikeCssDeclaration(line) && !looksLikeCssRule(line))
		.join('\n')
		.trim();
	if (!cleaned) return '';
	return cleaned
		.split(/\n{2,}/)
		.map((paragraph) => `<p>${escapeHtml(decodeBasicEntities(paragraph)).replace(/\n/g, '<br>')}</p>`)
		.join('');
}

function sanitizeEmailHtml(value: string): string {
	const withoutUnsafeBlocks = value
		.replace(/<!doctype[^>]*>/gi, ' ')
		.replace(/<!--[\s\S]*?-->/g, ' ')
		.replace(/<style\b[^>]*>[\s\S]*?<\/style>/gi, ' ')
		.replace(/<script\b[^>]*>[\s\S]*?<\/script>/gi, ' ')
		.replace(/<head\b[^>]*>[\s\S]*?<\/head>/gi, ' ')
		.replace(/<title\b[^>]*>[\s\S]*?<\/title>/gi, ' ')
		.replace(/<meta\b[^>]*>/gi, ' ')
		.replace(/<link\b[^>]*>/gi, ' ')
		.replace(/<iframe\b[^>]*>[\s\S]*?<\/iframe>/gi, ' ')
		.replace(/<object\b[^>]*>[\s\S]*?<\/object>/gi, ' ')
		.replace(/<embed\b[^>]*>/gi, ' ')
		.replace(/<svg\b[^>]*>[\s\S]*?<\/svg>/gi, ' ')
		.replace(/<math\b[^>]*>[\s\S]*?<\/math>/gi, ' ')
		.replace(/<form\b[^>]*>[\s\S]*?<\/form>/gi, ' ');
	const output: string[] = [];
	let lastIndex = 0;
	const tagPattern = /<[^>]*>/g;
	let match: RegExpExecArray | null;

	while ((match = tagPattern.exec(withoutUnsafeBlocks)) !== null) {
		output.push(escapeHtml(decodeBasicEntities(withoutUnsafeBlocks.slice(lastIndex, match.index))));
		output.push(sanitizeEmailTag(match[0]));
		lastIndex = match.index + match[0].length;
	}
	output.push(escapeHtml(decodeBasicEntities(withoutUnsafeBlocks.slice(lastIndex))));

	const sanitized = cleanupSanitizedEmailHtml(output.join('').replace(/[ \t]{2,}/g, ' ').trim());
	return sanitized || '<p></p>';
}

const allowedEmailTags = new Set([
	'a',
	'blockquote',
	'br',
	'code',
	'div',
	'em',
	'h1',
	'h2',
	'h3',
	'h4',
	'h5',
	'h6',
	'hr',
	'li',
	'ol',
	'p',
	'pre',
	'small',
	'span',
	'strong',
	'table',
	'tbody',
	'td',
	'th',
	'thead',
	'tr',
	'u',
	'ul'
]);

const voidEmailTags = new Set(['br', 'hr']);

function sanitizeEmailTag(rawTag: string): string {
	const parsed = rawTag.match(/^<\s*(\/)?\s*([a-zA-Z][a-zA-Z0-9:-]*)([\s\S]*?)(\/?)\s*>$/);
	if (!parsed) return '';
	const isClosing = Boolean(parsed[1]);
	const originalTagName = parsed[2].toLowerCase();
	const tagName = normalizedEmailTagName(originalTagName);
	if (!allowedEmailTags.has(tagName)) {
		if (originalTagName === 'img') {
			const alt = readHtmlAttribute(rawTag, 'alt');
			return alt ? `<span class="mail-image-placeholder">${escapeHtml(decodeBasicEntities(alt))}</span>` : '';
		}
		return '';
	}
	if (voidEmailTags.has(tagName)) {
		return `<${tagName}>`;
	}
	if (isClosing) {
		return `</${tagName}>`;
	}
	return `<${tagName}${sanitizeEmailAttributes(tagName, rawTag)}>`;
}

function normalizedEmailTagName(tagName: string): string {
	if (tagName === 'b') return 'strong';
	if (tagName === 'i') return 'em';
	if (tagName === 'font') return 'span';
	return tagName;
}

function sanitizeEmailAttributes(tagName: string, rawTag: string): string {
	const attributes: string[] = [];
	if (tagName === 'a') {
		const href = readHtmlAttribute(rawTag, 'href');
		if (href && isSafeEmailHref(href)) {
			attributes.push(`href="${escapeHtml(decodeBasicEntities(href))}"`);
			attributes.push('target="_blank"');
			attributes.push('rel="noopener noreferrer"');
		}
	}
	if (tagName === 'td' || tagName === 'th') {
		for (const name of ['colspan', 'rowspan']) {
			const value = readHtmlAttribute(rawTag, name);
			if (value && /^\d{1,2}$/.test(value) && Number(value) > 0 && Number(value) <= 20) {
				attributes.push(`${name}="${value}"`);
			}
		}
	}
	return attributes.length ? ` ${attributes.join(' ')}` : '';
}

function readHtmlAttribute(rawTag: string, name: string): string | null {
	const pattern = new RegExp(`${name}\\s*=\\s*(?:"([^"]*)"|'([^']*)'|([^\\s"'=<>` + '`' + `]+))`, 'i');
	const match = rawTag.match(pattern);
	return match?.[1] ?? match?.[2] ?? match?.[3] ?? null;
}

function isSafeEmailHref(value: string): boolean {
	const decoded = decodeBasicEntities(value).trim();
	return /^(https?:|mailto:|tel:)/i.test(decoded);
}

function cleanupSanitizedEmailHtml(value: string): string {
	let previous = '';
	let cleaned = value;
	while (cleaned !== previous) {
		previous = cleaned;
		cleaned = cleaned
			.replace(/<a\b[^>]*>\s*<\/a>/gi, '')
			.replace(/<(span|strong|em|small)\b[^>]*>\s*<\/\1>/gi, '');
	}
	return cleaned.trim();
}

function looksLikeHtml(value: string): boolean {
	return /<\/?(?:html|body|div|p|span|table|tr|td|th|br|strong|b|em|i|a|style|head)\b[\s\S]*?>/i.test(value);
}

function looksLikeEscapedHtml(value: string): boolean {
	return /&lt;\/?(?:html|body|div|p|span|table|tr|td|th|br|strong|b|em|i|a|style|head)\b[\s\S]*?&gt;/i.test(value);
}

function stripLeadingMimeHeaderBlock(value: string): string {
	const lines = value.split(/\r?\n/);
	let sawHeader = false;
	let index = 0;
	while (index < lines.length) {
		const line = lines[index].trim();
		if (looksLikeMimeHeader(line)) {
			sawHeader = true;
			index += 1;
			continue;
		}
		if (sawHeader && !line) {
			index += 1;
			break;
		}
		break;
	}
	return sawHeader ? lines.slice(index).join('\n') : value;
}

function escapeHtml(value: string): string {
	return value
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;')
		.replace(/'/g, '&#39;');
}

function looksLikeMimeHeader(line: string): boolean {
	return /^(content-type|mime-version|content-transfer-encoding|received|dkim-signature|message-id|from|to|subject|date)\s*:/i.test(line);
}

function looksLikeCssDeclaration(line: string): boolean {
	const declarations = line.split(';').map((part) => part.trim()).filter(Boolean);
	if (!declarations.length) return false;
	const cssLike = declarations.filter((part) =>
		/^(margin|padding|font|font-family|font-size|line-height|color|background|border|width|height|display|box-sizing|min-width|max-width|table-layout|border-collapse)\s*:/i.test(part)
	);
	return cssLike.length > 0 && cssLike.length === declarations.length;
}

function looksLikeCssRule(line: string): boolean {
	return /\{[^}]*\b(margin|padding|font|font-family|font-size|line-height|color|background|border|width|height|display|box-sizing|min-width|max-width|table-layout|border-collapse)\s*:/i.test(line);
}

function decodeBasicEntities(value: string): string {
	return value
		.replace(/&zwnj;|&zwj;/gi, ' ')
		.replace(/&shy;/gi, '')
		.replace(/&nbsp;/g, ' ')
		.replace(/&amp;/g, '&')
		.replace(/&lt;/g, '<')
		.replace(/&gt;/g, '>')
		.replace(/&quot;/g, '"')
		.replace(/&#39;/g, "'")
		.replace(/&shy;/gi, '')
		.replace(/&#x([0-9a-f]+);/gi, (_, code: string) => entityCodeToText(Number.parseInt(code, 16)))
		.replace(/&#(\d+);/g, (_, code: string) => entityCodeToText(Number.parseInt(code, 10)));
}

function entityCodeToText(code: number): string {
	if (!Number.isFinite(code)) return ' ';
	if (code === 8204 || code === 8205 || code === 65279) return ' ';
	if (code < 32 || code === 127) return ' ';
	try {
		const value = String.fromCodePoint(code);
		return value.trim() ? value : ' ';
	} catch {
		return ' ';
	}
}

function truncatePreview(value: string, limit = 140): string {
	if (value.length <= limit) return value;
	return `${value.slice(0, Math.max(0, limit - 3)).trimEnd()}...`;
}
