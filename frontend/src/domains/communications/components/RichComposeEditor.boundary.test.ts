import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const componentPath = resolve(dirname(fileURLToPath(import.meta.url)), 'RichComposeEditor.vue')
const schemaPath = resolve(dirname(fileURLToPath(import.meta.url)), 'richComposeExtensions.ts')

describe('RichComposeEditor boundaries', () => {
  it('uses TipTap runtime instead of browser execCommand contenteditable editing', () => {
    const source = readFileSync(componentPath, 'utf8')

    expect(source).toContain("import { richComposeExtensions } from './richComposeExtensions'")
    expect(source).toContain('useEditor')
    expect(source).toContain('EditorContent')
    expect(source).toContain('richComposeExtensions')
    expect(source).not.toContain('Node.create')
    expect(source).not.toContain('Mark.create')
    expect(source).not.toContain('document.execCommand')
    expect(source).not.toContain('contenteditable="true"')
  })

  it('keeps compose formatting commands scoped to supported mail-safe controls', () => {
    const source = readFileSync(componentPath, 'utf8')

    expect(source).toContain('linkHref')
    expect(source).toContain("runCommand('paragraph')")
    expect(source).toContain("runCommand('heading2')")
    expect(source).toContain("runCommand('heading3')")
    expect(source).toContain("runCommand('alignLeft')")
    expect(source).toContain("runCommand('alignCenter')")
    expect(source).toContain("runCommand('alignRight')")
    expect(source).toContain('updateAttributes')
    expect(source).toContain("runCommand('bold')")
    expect(source).toContain("runCommand('italic')")
    expect(source).toContain("runCommand('bulletList')")
    expect(source).toContain("runCommand('orderedList')")
    expect(source).toContain("runCommand('blockquote')")
    expect(source).toContain("runCommand('link')")
    expect(source).toContain("runCommand('unlink')")
  })

  it('keeps the local TipTap schema in a focused mail-safe extension module', () => {
    const source = readFileSync(schemaPath, 'utf8')

    expect(source).toContain("from '@tiptap/vue-3'")
    expect(source).toContain('RichHeading')
    expect(source).toContain('RichLink')
    expect(source).toContain('RichOrderedList')
    expect(source).toContain('RichBlockquote')
    expect(source).toContain('normalizeMailComposeLinkHref')
    expect(source).toContain('normalizeMailComposeTextAlign')
    expect(source).toContain('getSafeTextAlignAttributes')
    expect(source).toContain("rel: 'noopener noreferrer'")
    expect(source).toContain("target: '_blank'")
    expect(source).toContain('export const richComposeExtensions')
  })

  it('intercepts pasted and dropped HTML before TipTap inserts it into the draft', () => {
    const source = readFileSync(componentPath, 'utf8')

    expect(source).toContain('sanitizeMailComposePastedHtml')
    expect(source).toContain('handlePaste')
    expect(source).toContain('handleDrop')
    expect(source).toContain('event.preventDefault()')
    expect(source).toContain('insertContent(sanitizeMailComposePastedHtml')
  })

  it('emits dropped attachment files instead of inserting them into rich HTML', () => {
    const source = readFileSync(componentPath, 'utf8')

    expect(source).toContain("'attachments-dropped': [files: File[]]")
    expect(source).toContain("emit('attachments-dropped'")
    expect(source).toContain('Array.from(event.dataTransfer?.files')
  })
})
