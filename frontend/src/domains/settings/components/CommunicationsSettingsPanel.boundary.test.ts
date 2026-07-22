import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationsSettingsPanel boundary', () => {
  it('keeps mail controls in Settings and routes mutations through shared mail sync', () => {
    const panel = readFileSync(new URL('./CommunicationsSettingsPanel.vue', import.meta.url), 'utf8')
    const surface = readFileSync(
      new URL('../queries/useCommunicationsSettingsSurface.ts', import.meta.url),
      'utf8'
    )

    expect(panel).toContain('useCommunicationsSettingsPanelController')
    expect(panel).toContain('Provider folders & labels')
    expect(panel).toContain('surface.providerResources.value')
    expect(panel).toContain('surface.localFolders.value')
    expect(panel).toContain('External content access')
    expect(panel).toContain('handleResourceRoleInput')
    expect(panel).toContain('handleResourceLocalFolderInput')
    expect(panel).toContain('handleContentEgressBodyToggle')
    expect(panel).toContain('handleContentEgressAttachmentsToggle')
    expect(panel).toContain('handleContentEgressExtractedTextToggle')
    expect(panel).toContain('handleSaveSensitiveForwardingPolicy')
    expect(panel).toContain('handleRemoveSelectedSensitiveForwardingPolicy')
    expect(panel).not.toContain('saveSensitiveForwardingPolicy')
    expect(panel).not.toContain('removeSelectedSensitiveForwardingPolicy')
    expect(panel).not.toContain("../../communications/")
    expect(panel).not.toContain('surface.updateProviderResourceRole')
    expect(panel).not.toContain('surface.updateProviderResourceLocalFolder')
    expect(panel).not.toContain('surface.updateSelectedMailContentEgress')

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
