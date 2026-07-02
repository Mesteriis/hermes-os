# RichTextEditor

Shared, provider-neutral rich text surface for context notes, local drafts and evidence snippets.

The component intentionally exposes a compact semantic toolbar: paragraph, headings, quote, ordered and unordered lists, emphasis, nuance, underline, strike, inline code, evidence link, code block, divider and clear formatting. It emits sanitized HTML through `v-model` and leaves persistence, validation and provider actions to the owning domain surface.

Use `HtmlPreview` when the caller needs to render the emitted HTML next to the editor.
