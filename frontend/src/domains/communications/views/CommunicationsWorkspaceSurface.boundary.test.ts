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
    const communicationSurfaceSource = readFileSync(
      new URL('../queries/communicationChannelSurface.ts', import.meta.url),
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
    const zulipSurfaceSource = readFileSync(
      new URL('../queries/useZulipCommunicationsSurface.ts', import.meta.url),
      'utf8'
    )
    const slackSurfaceSource = readFileSync(
      new URL('../queries/useSlackCommunicationsSurface.ts', import.meta.url),
      'utf8'
    )
    const discordSurfaceSource = readFileSync(
      new URL('../queries/useDiscordCommunicationsSurface.ts', import.meta.url),
      'utf8'
    )
    const mattermostSurfaceSource = readFileSync(
      new URL('../queries/useMattermostCommunicationsSurface.ts', import.meta.url),
      'utf8'
    )
    const callsSurfaceSource = readFileSync(
      new URL('../queries/useCallsCommunicationsSurface.ts', import.meta.url),
      'utf8'
    )
    const meetingsSurfaceSource = readFileSync(
      new URL('../queries/useMeetingsCommunicationsSurface.ts', import.meta.url),
      'utf8'
    )
    const timelineSurfaceSource = readFileSync(
      new URL('../queries/useCommunicationTimelineSurface.ts', import.meta.url),
      'utf8'
    )

    expect(appSurfaceSource).toContain('useCommunicationsWorkspaceSurface')
    expect(appSurfaceSource).toContain('childSurfaces: communications.childSurfaces')

    expect(workspaceSurfaceSource).toContain('useMailCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useTelegramCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useWhatsappCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useZulipCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useSlackCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useDiscordCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useMattermostCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useCallsCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useMeetingsCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useCommunicationTimelineSurface')
    expect(workspaceSurfaceSource).toContain('createCommunicationSurface')
    expect(workspaceSurfaceSource).toContain("surfaceId: 'communications'")
    expect(workspaceSurfaceSource).toContain('commonCapabilities')
    expect(workspaceSurfaceSource).toContain('subSurfaces')

    expect(communicationSurfaceSource).toContain('CommunicationSurface')
    expect(communicationSurfaceSource).toContain('CommunicationSubSurface')
    expect(communicationSurfaceSource).toContain('CommunicationSurfaceCapabilityGroup')
    expect(communicationSurfaceSource).toContain('communicationSurfaceChild')

    expect(mailSurfaceSource).toContain("businessQueryRoot: ['communications', 'mail']")
    expect(mailSurfaceSource).toContain('useCommunicationsPageSurface.ts')
    expect(telegramSurfaceSource).toContain('telegramBusinessQueryKeys')
    expect(telegramSurfaceSource).toContain("businessQueryRoot: ['communications', 'telegram']")
    expect(whatsappSurfaceSource).toContain('whatsappBusinessQueryKeys')
    expect(whatsappSurfaceSource).toContain("businessQueryRoot: ['communications', 'whatsapp']")
    expect(zulipSurfaceSource).toContain("channelId: 'zulip'")
    expect(zulipSurfaceSource).toContain("businessQueryRoot: ['communications', 'channels']")
    expect(zulipSurfaceSource).toContain("runtimeQueryRoot: ['integrations', 'zulip', 'runtime']")
    expect(zulipSurfaceSource).toContain('send_stream_message')
    expect(zulipSurfaceSource).toContain('signal.raw.zulip.message.observed')
    expect(zulipSurfaceSource).toContain('signal.accepted.zulip.message')
    expect(slackSurfaceSource).toContain("channelId: 'slack'")
    expect(slackSurfaceSource).toContain("status: 'facade'")
    expect(discordSurfaceSource).toContain("channelId: 'discord'")
    expect(discordSurfaceSource).toContain("status: 'facade'")
    expect(mattermostSurfaceSource).toContain("channelId: 'mattermost'")
    expect(mattermostSurfaceSource).toContain("status: 'facade'")
    expect(callsSurfaceSource).toContain("channelId: 'calls'")
    expect(callsSurfaceSource).toContain("businessQueryRoot: ['communications', 'calls']")
    expect(callsSurfaceSource).toContain('communications.calls.recordings')
    expect(meetingsSurfaceSource).toContain("channelId: 'meetings'")
    expect(meetingsSurfaceSource).toContain("status: 'facade'")
    expect(meetingsSurfaceSource).toContain('communications.meetings.permanent_rooms')
    expect(timelineSurfaceSource).toContain("channelId: 'communications-timeline'")
    expect(timelineSurfaceSource).toContain("businessQueryRoot: ['communications', 'timeline']")

    expect(workspaceSurfaceSource).not.toContain('frontend/src/integrations')
    expect(mailSurfaceSource).not.toContain('frontend/src/integrations')
    expect(telegramSurfaceSource).not.toContain('frontend/src/integrations')
    expect(whatsappSurfaceSource).not.toContain('frontend/src/integrations')
    expect(zulipSurfaceSource).not.toContain('frontend/src/integrations')
    expect(slackSurfaceSource).not.toContain('frontend/src/integrations')
    expect(discordSurfaceSource).not.toContain('frontend/src/integrations')
    expect(mattermostSurfaceSource).not.toContain('frontend/src/integrations')
    expect(callsSurfaceSource).not.toContain('frontend/src/integrations')
    expect(meetingsSurfaceSource).not.toContain('frontend/src/integrations')
    expect(timelineSurfaceSource).not.toContain('frontend/src/integrations')
  })
})
