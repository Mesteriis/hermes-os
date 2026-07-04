import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('Communications workspace surface', () => {
  it('keeps Mail, Telegram and WhatsApp under one Communications facade', () => {
    const appSurfaceSource = readFileSync(
      new URL('../../../app/queries/useCommunicationsViewSurface.ts', import.meta.url),
      'utf8'
    )
    const workspaceSurfaceSource = readFileSync(
      new URL('../queries/useCommunicationsWorkspaceSurface.ts', import.meta.url),
      'utf8'
    )
    const mailSurfaceSource = readFileSync(
      new URL('../queries/useMailCommunicationsSurface.ts', import.meta.url),
      'utf8'
    )
    const telegramSurfaceSource = readFileSync(
      new URL('../queries/useTelegramCommunicationsSurface.ts', import.meta.url),
      'utf8'
    )
    const whatsappSurfaceSource = readFileSync(
      new URL('../queries/useWhatsappCommunicationsSurface.ts', import.meta.url),
      'utf8'
    )

    expect(appSurfaceSource).toContain('useCommunicationsWorkspaceSurface')
    expect(appSurfaceSource).toContain('childSurfaces: communications.childSurfaces')

    expect(workspaceSurfaceSource).toContain('useMailCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useTelegramCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useWhatsappCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain("surfaceId: 'communications'")

    expect(mailSurfaceSource).toContain("businessQueryRoot: ['communications', 'mail']")
    expect(mailSurfaceSource).toContain('useCommunicationsPageSurface.ts')
    expect(telegramSurfaceSource).toContain('telegramBusinessQueryKeys')
    expect(telegramSurfaceSource).toContain("businessQueryRoot: ['communications', 'telegram']")
    expect(whatsappSurfaceSource).toContain('whatsappBusinessQueryKeys')
    expect(whatsappSurfaceSource).toContain("businessQueryRoot: ['communications', 'whatsapp']")

    expect(workspaceSurfaceSource).not.toContain('frontend/src/integrations')
    expect(mailSurfaceSource).not.toContain('frontend/src/integrations')
    expect(telegramSurfaceSource).not.toContain('frontend/src/integrations')
    expect(whatsappSurfaceSource).not.toContain('frontend/src/integrations')
  })
})
