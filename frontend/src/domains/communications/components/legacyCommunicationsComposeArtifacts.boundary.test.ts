import { existsSync, readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const componentDir = dirname(fileURLToPath(import.meta.url))
const formsDir = resolve(componentDir, '../forms')

const removedComposeFiles = [
  'ComposeDrawer.vue',
  'ComposeSignaturePicker.vue',
  'ComposeTemplatePicker.vue',
  'RichComposeEditor.vue',
  'TemplateRecipientMappingPanel.vue',
  'TemplateSaveForm.vue',
  'ComposeDrawer.css',
  'ComposeTemplatePicker.css'
]

function readComponentArtifact(relativePath: string): string {
  return readFileSync(resolve(componentDir, relativePath), 'utf8')
}

function readFormArtifact(relativePath: string): string {
  return readFileSync(resolve(formsDir, relativePath), 'utf8')
}

describe('legacy communications compose artifacts', () => {
  it('removes the Vue compose render layer and its component-owned stylesheets', () => {
    for (const relativePath of removedComposeFiles) {
      expect(existsSync(resolve(componentDir, relativePath))).toBe(false)
    }
  })

  it('preserves compose draft autosave and validation as TypeScript artifacts', () => {
    const autosaveSource = readFormArtifact('composeDraftAutosave.ts')
    const validationSource = readFormArtifact('composeValidation.ts')

    expect(autosaveSource).toContain('buildComposeDraftPayload')
    expect(autosaveSource).toContain('composeDraftHasAutosaveContent')
    expect(autosaveSource).toContain('datetimeLocalToIso')
    expect(autosaveSource).toContain('useComposeDraftAutosave')
    expect(validationSource).toContain('splitComposeRecipients')
    expect(validationSource).toContain('composeSendSchema')
    expect(validationSource).toContain('composeVeeValidationSchema')
    expect(validationSource).toContain('useComposeValidation')
  })

  it('preserves template and rich-text business helpers outside the deleted Vue layer', () => {
    const templateLibrarySource = readComponentArtifact('templateLibrary.ts')
    const richComposeHtmlSource = readComponentArtifact('richComposeHtml.ts')
    const richComposeExtensionsSource = readComponentArtifact('richComposeExtensions.ts')

    expect(templateLibrarySource).toContain('templateLibraryCategoryOptions')
    expect(templateLibrarySource).toContain('deriveTemplateLibraryCategories')
    expect(templateLibrarySource).toContain('applyTemplateRecipientMapping')
    expect(templateLibrarySource).toContain('buildTemplateRecipientPreviewRows')
    expect(templateLibrarySource).toContain('recipientPreviewSummary')
    expect(templateLibrarySource).toContain('suggestTemplateSaveName')
    expect(richComposeHtmlSource).toContain('plainTextToComposeHtml')
    expect(richComposeHtmlSource).toContain('htmlToComposePlainText')
    expect(richComposeHtmlSource).toContain('appendHtmlSignature')
    expect(richComposeHtmlSource).toContain('sanitizeMailComposePastedHtml')
    expect(richComposeExtensionsSource).toContain('normalizeMailComposeLinkHref')
    expect(richComposeExtensionsSource).toContain('normalizeMailComposeTextAlign')
    expect(richComposeExtensionsSource).toContain('const RichHeading')
    expect(richComposeExtensionsSource).toContain('const RichLink')
    expect(richComposeExtensionsSource).toContain('export const richComposeExtensions')
  })
})
