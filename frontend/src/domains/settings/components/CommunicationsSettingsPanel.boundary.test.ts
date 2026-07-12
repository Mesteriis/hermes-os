import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationsSettingsPanel boundary', () => {
  it('keeps mail controls in Settings and routes mutations through shared mail sync', () => {
    const panel = readFileSync(new URL('./CommunicationsSettingsPanel.vue', import.meta.url), 'utf8')
    const surface = readFileSync(
      new URL('../queries/useCommunicationsSettingsSurface.ts', import.meta.url),
      'utf8'
    )

    expect(panel).toContain("from '../../../shared/mailSync/providerResources'")
    expect(panel).toContain('Provider folders & labels')
    expect(panel).toContain('surface.providerResources.value')
    expect(panel).toContain('surface.updateProviderResourceRole(resource, semanticRoleValue($event))')
    expect(panel).toContain('surface.localFolders.value')
    expect(panel).toContain('surface.updateProviderResourceLocalFolder(resource, localFolderValue($event))')
    expect(panel).toContain('External content access')
    expect(panel).toContain("surface.updateSelectedMailContentEgress('body', eventChecked($event))")
    expect(panel).toContain("surface.updateSelectedMailContentEgress('attachments', eventChecked($event))")
    expect(panel).toContain("surface.updateSelectedMailContentEgress('extracted_text', eventChecked($event))")
    expect(panel).not.toContain("../../communications/")

    expect(surface).toContain('useMailProviderResourcesQuery')
    expect(surface).toContain('useMailLocalFoldersQuery')
    expect(surface).toContain('useUpdateMailProviderResourceMappingMutation')
    expect(surface).toContain('useMailContentEgressSettingsQuery')
    expect(surface).toContain('useUpdateMailContentEgressSettingsMutation')
    expect(surface).toContain('updateSelectedMailContentEgress')
    expect(surface).toContain('updateProviderResourceRole')
    expect(surface).toContain('updateProviderResourceLocalFolder')
  })
})
