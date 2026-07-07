import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('Communications workspace surface', () => {
  it('keeps Mail, Telegram and WhatsApp under one Communications facade', () => {
    const appSurfaceSource = readFileSync(
      new URL(
        '../../../app/queries/useCommunicationsViewSurface.ts',
        import.meta.url
      ),
      'utf8'
    )
    const workspaceSurfaceSource = readFileSync(
      new URL(
        '../queries/useCommunicationsWorkspaceSurface.ts',
        import.meta.url
      ),
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
      new URL(
        '../queries/useTelegramCommunicationsSurface.ts',
        import.meta.url
      ),
      'utf8'
    )
    const whatsappSurfaceSource = readFileSync(
      new URL(
        '../queries/useWhatsappCommunicationsSurface.ts',
        import.meta.url
      ),
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
      new URL(
        '../queries/useMattermostCommunicationsSurface.ts',
        import.meta.url
      ),
      'utf8'
    )
    const callsSurfaceSource = readFileSync(
      new URL('../queries/useCallsCommunicationsSurface.ts', import.meta.url),
      'utf8'
    )
    const meetingsSurfaceSource = readFileSync(
      new URL(
        '../queries/useMeetingsCommunicationsSurface.ts',
        import.meta.url
      ),
      'utf8'
    )
    const timelineSurfaceSource = readFileSync(
      new URL('../queries/useCommunicationTimelineSurface.ts', import.meta.url),
      'utf8'
    )

    expect(appSurfaceSource).toContain('useCommunicationsWorkspaceSurface')
    expect(appSurfaceSource).toContain(
      'childSurfaces: communications.childSurfaces'
    )
    expect(appSurfaceSource).toContain("status: 'active'")

    expect(workspaceSurfaceSource).toContain('useMailCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useTelegramCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useWhatsappCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useZulipCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useSlackCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useDiscordCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain(
      'useMattermostCommunicationsSurface'
    )
    expect(workspaceSurfaceSource).toContain('useCallsCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useMeetingsCommunicationsSurface')
    expect(workspaceSurfaceSource).toContain('useCommunicationTimelineSurface')
    expect(workspaceSurfaceSource).toContain('createCommunicationSurface')
    expect(workspaceSurfaceSource).toContain("surfaceId: 'communications'")
    expect(workspaceSurfaceSource).toContain('commonCapabilities')
    expect(workspaceSurfaceSource).toContain('subSurfaces')

    expect(communicationSurfaceSource).toContain('CommunicationSurface')
    expect(communicationSurfaceSource).toContain('CommunicationSubSurface')
    expect(communicationSurfaceSource).toContain(
      'CommunicationSurfaceCapabilityGroup'
    )
    expect(communicationSurfaceSource).toContain('communicationSurfaceChild')

    expect(mailSurfaceSource).toContain(
      "businessQueryRoot: ['communications', 'mail']"
    )
    expect(mailSurfaceSource).toContain('useCommunicationsPageSurface.ts')
    expect(telegramSurfaceSource).toContain('telegramBusinessQueryKeys')
    expect(telegramSurfaceSource).toContain(
      "businessQueryRoot: ['communications', 'telegram']"
    )
    expect(whatsappSurfaceSource).toContain('whatsappBusinessQueryKeys')
    expect(whatsappSurfaceSource).toContain(
      "businessQueryRoot: ['communications', 'whatsapp']"
    )
    expect(zulipSurfaceSource).toContain("channelId: 'zulip'")
    expect(zulipSurfaceSource).toContain(
      "businessQueryRoot: ['communications', 'channels']"
    )
    expect(zulipSurfaceSource).toContain(
      "runtimeQueryRoot: ['integrations', 'zulip', 'runtime']"
    )
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
    expect(callsSurfaceSource).toContain(
      "businessQueryRoot: ['communications', 'calls']"
    )
    expect(callsSurfaceSource).toContain('communications.calls.recordings')
    expect(meetingsSurfaceSource).toContain("channelId: 'meetings'")
    expect(meetingsSurfaceSource).toContain("status: 'facade'")
    expect(meetingsSurfaceSource).toContain(
      'communications.meetings.permanent_rooms'
    )
    expect(timelineSurfaceSource).toContain(
      "channelId: 'communications-timeline'"
    )
    expect(timelineSurfaceSource).toContain(
      "businessQueryRoot: ['communications', 'timeline']"
    )

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

  it('routes Communications channel leaves to existing mail and messenger workspaces', () => {
    const viewSource = readFileSync(
      new URL('./CommunicationsWorkspaceView.vue', import.meta.url),
      'utf8'
    )
    const viewSurfaceSource = readFileSync(
      new URL(
        '../queries/useCommunicationsWorkspaceViewSurface.ts',
        import.meta.url
      ),
      'utf8'
    )
    const mailWorkspaceModelsSource = readFileSync(
      new URL('../queries/communicationMailWorkspaceModels.ts', import.meta.url),
      'utf8'
    )
    const mailListSource = readFileSync(
      new URL('../components/mail/MailList.vue', import.meta.url),
      'utf8'
    )
    const mailWorkspaceSource = readFileSync(
      new URL('../components/mail/MailWorkspace.vue', import.meta.url),
      'utf8'
    )
    const mailSyncProgressSource = readFileSync(
      new URL('../components/mail/MailSyncProgress.vue', import.meta.url),
      'utf8'
    )
    const pageSurfaceSource = readFileSync(
      new URL('../queries/useCommunicationsPageSurface.ts', import.meta.url),
      'utf8'
    )
    const accountApiSource = readFileSync(
      new URL('../api/accountApi.ts', import.meta.url),
      'utf8'
    )
    const mailAccountQueriesSource = readFileSync(
      new URL('../queries/mailAccountQueries.ts', import.meta.url),
      'utf8'
    )
    const mailCoreQueriesSource = readFileSync(
      new URL('../queries/mailCoreQueries.ts', import.meta.url),
      'utf8'
    )
    const mailWorkspaceQueriesSource = readFileSync(
      new URL('../queries/mailWorkspaceQueries.ts', import.meta.url),
      'utf8'
    )
    const mailListViewsSource = readFileSync(
      new URL('../components/mail/mailListViews.ts', import.meta.url),
      'utf8'
    )
    const mailComposeOptionsSource = readFileSync(
      new URL('../components/mail/mailComposeOptions.ts', import.meta.url),
      'utf8'
    )
    const communicationDomainElementsCss = readFileSync(
      new URL('../components/communicationDomainElements.css', import.meta.url),
      'utf8'
    )

    expect(viewSource).toContain('MailWorkspace')
    expect(viewSource).toContain(
      ':has-more-items="surface.pageSurface.hasVisibleNextPage.value"'
    )
    expect(viewSource).toContain(
      ':is-loading-more="surface.pageSurface.isFetchingVisibleNextPage.value"'
    )
    expect(viewSource).toContain(
      ':search-query="surface.pageSurface.store.messageSearchQuery"'
    )
    expect(viewSource).toContain(
      ':compose-account-options="surface.pageSurface.mailComposeAccountOptions.value"'
    )
    expect(viewSource).toContain(':sync-status="surface.mailSyncStatus.value"')
    expect(viewSource).toContain(
      '@load-more="surface.pageSurface.handleLoadMoreMessages"'
    )
    expect(viewSource).toContain(
      '@update-search-query="surface.pageSurface.handleSearchQueryUpdate"'
    )
    expect(viewSource).toContain('MessengerWorkspace')
    expect(viewSource).toContain("surface.activeChannelId.value === 'mail'")
    expect(viewSource).toContain("surface.activeChannelId.value === 'telegram'")
    expect(viewSource).not.toContain('communication-workspace-menu')
    expect(viewSource).not.toContain('CommunicationWorkspaceShell')
    expect(viewSource).not.toContain('CommunicationWorkspaceOverview')
    expect(viewSurfaceSource).toContain('useTelegramChatsQuery')
    expect(viewSurfaceSource).toContain('useTelegramMessagesQuery')
    expect(viewSurfaceSource).toContain('useWhatsappBusinessConversationsQuery')
    expect(viewSurfaceSource).toContain('useWhatsappBusinessMessagesQuery')
    expect(viewSurfaceSource).toContain('routeToChannelId')
    expect(viewSurfaceSource).toContain('useNotificationsStore')
    expect(viewSurfaceSource).toContain('pendingNotificationTarget')
    expect(viewSurfaceSource).toContain("notification?.targetView !== 'communications-mail'")
    expect(viewSurfaceSource).toContain('pageSurface.store.selectMessageId(notification.targetId)')
    expect(viewSurfaceSource).toContain("pageSurface.store.setActiveMessageContextTab('message')")
    expect(viewSurfaceSource).toContain('consumePendingNotificationTarget')
    expect(viewSurfaceSource).toContain('mailSyncStatus')
    expect(viewSurfaceSource).toContain('mailSyncStatusIsActive')
    expect(viewSurfaceSource).toContain(
      "pageSurface.store.setLocalStateFilter('all')"
    )
    expect(viewSurfaceSource).toContain("pageSurface.store.setStateFilter('')")
    expect(viewSurfaceSource).toContain('mailItem(')
    expect(mailWorkspaceModelsSource).toContain('message.message_metadata.mailbox')
    expect(mailWorkspaceModelsSource).toContain("normalized.includes('junk')")
    expect(mailWorkspaceModelsSource).toContain("normalized.includes('spam')")
    expect(mailWorkspaceModelsSource).toContain('mailboxIsSent(mailbox)')
    expect(mailWorkspaceModelsSource).toContain('mailboxIsDrafts(mailbox)')
    expect(mailWorkspaceModelsSource).toContain('mailboxIsTrash(mailbox)')
    expect(mailListSource).toContain('MailSyncProgress')
    expect(mailListSource).toContain(':status="syncStatus"')
    expect(mailListSource).toContain('hasMoreItems?: boolean')
    expect(mailListSource).toContain("'load-more': []")
    expect(mailListSource).toContain('@scroll="handleBodyScroll"')
    expect(mailListSource).toContain("emit('load-more')")
    expect(mailListSource).toContain('mail-list-load-more')
    expect(mailListSource).toContain(
      'mailListTreeSelectOptions(listItems.value, savedFilterOptions, t, Boolean(props.hasMoreItems))'
    )
    expect(mailListViewsSource).toContain('hasMoreItems = false')
    expect(mailListViewsSource).toContain("const suffix = hasMoreItems ? '+' : ''")
    expect(mailWorkspaceSource).toContain(':has-more-items="hasMoreItems"')
    expect(mailWorkspaceSource).toContain(':is-loading-more="isLoadingMore"')
    expect(mailWorkspaceSource).toContain('@load-more="emit(\'load-more\')"')
    expect(mailWorkspaceSource).toContain('<Dialog')
    expect(mailWorkspaceSource).toContain('content-class="mail-compose-dialog"')
    expect(mailWorkspaceSource).toContain('@update:open="handleComposeDialogOpenChange"')
    expect(mailWorkspaceSource).toContain('RichTextEditor')
    expect(mailWorkspaceSource).toContain('composeAccountOptions')
    expect(mailWorkspaceSource).toContain('plainTextToComposeHtml')
    expect(mailWorkspaceSource).toContain('htmlToComposePlainText')
    expect(mailWorkspaceSource).toContain('handleComposeBodyHtmlChange')
    expect(mailWorkspaceSource).toContain('mail-compose-panel__field--from')
    expect(mailWorkspaceSource).toContain('composeSendAccountOptions')
    expect(mailWorkspaceSource).toContain(':disabled="!account.can_send"')
    expect(mailWorkspaceSource).toContain('activeComposePanel')
    expect(mailWorkspaceSource).toContain('toggleComposeEdgePanel')
    expect(mailWorkspaceSource).toContain('closeComposeEdgePanels')
    expect(mailWorkspaceSource).toContain('compose-edge-panel--left')
    expect(mailWorkspaceSource).toContain('compose-edge-panel--right')
    expect(mailWorkspaceSource).toContain('mail-compose-stage')
    expect(mailWorkspaceSource).toContain('mail-compose-card')
    expect(mailComposeOptionsSource).toContain('ComposeEdgePanelId')
    expect(mailComposeOptionsSource).toContain('composeAiPanelActions')
    expect(mailComposeOptionsSource).toContain('composeContextPanelSections')
    expect(mailWorkspaceSource).not.toContain('{{ account.account_id }}')
    expect(communicationDomainElementsCss).toContain('.mail-compose-dialog.hermes-dialog-content')
    expect(pageSurfaceSource).toContain('handleLoadMoreMessages')
    expect(pageSurfaceSource).toContain('useEmailAccountsQuery')
    expect(pageSurfaceSource).toContain('mailComposeAccountOptions')
    expect(pageSurfaceSource).toContain('sendCapableMailComposeAccountOptions')
    expect(pageSurfaceSource).toContain('send_unavailable_reason')
    expect(pageSurfaceSource).not.toContain(
      '.filter((item) => item.capabilities.send)'
    )
    expect(pageSurfaceSource).toContain('getDefaultMailAccountId')
    expect(pageSurfaceSource).toContain('composeFormWithAvailableMailAccount')
    expect(accountApiSource).toContain('/api/v1/communications/email/accounts')
    expect(accountApiSource).not.toContain(['/api/v1/integrations', 'mail/accounts'].join('/'))
    expect(mailAccountQueriesSource).toContain('useEmailAccountsQuery')
    expect(pageSurfaceSource).not.toContain(
      'watch([hasNextPage, isFetchingNextPage, activeFolderId]'
    )
    expect(pageSurfaceSource).not.toContain(
      'watch([folderMail.hasNextPage, folderMail.isFetchingNextPage, activeFolderId]'
    )
    expect(mailCoreQueriesSource).toContain('const pageSize = 100')
    expect(mailWorkspaceQueriesSource).toContain('fetchFolderMessages(id, 100, pageParam)')
    expect(mailListSource.indexOf('communication-workspace-panel--inbox')).toBeLessThan(
      mailListSource.indexOf('<MailSyncProgress')
    )
    expect(mailListSource).toContain('syncProgressVisible')
    expect(mailListSource).toContain('mail-sync-progress-region')
    expect(mailListSource).toContain(':aria-hidden="!syncProgressVisible"')
    expect(mailListSource).toContain('@visibility-change="handleSyncProgressVisibilityChange"')
    expect(mailSyncProgressSource).toContain('v-if="status"')
    expect(mailSyncProgressSource).toContain("defineEmits<{")
    expect(mailSyncProgressSource).toContain("'visibility-change': [visible: boolean]")
    expect(mailSyncProgressSource).toContain(
      "status === 'recoverable_full_resync_needed'"
    )
    expect(mailSyncProgressSource).toContain('useCommunicationActionNotifications')
    expect(mailSyncProgressSource).toContain('mailSyncFailureKey')
    expect(mailSyncProgressSource).toContain('mail-sync-progress--exiting')
    expect(mailSyncProgressSource).toContain('failureKey.value !== null')
    expect(mailSyncProgressSource).toContain('mail-sync:${key}')
    expect(mailSyncProgressSource).toContain('return phaseLabel(status.phase)')
    expect(mailSyncProgressSource).not.toContain(
      '`${status.account_id} · ${phaseLabel(status.phase)}`'
    )
    expect(mailSyncProgressSource).toContain('mail-sync-progress__ambient')
    expect(mailSyncProgressSource).toContain('mail-sync-progress__orb')
    expect(communicationDomainElementsCss).toContain('.mail-sync-progress--exiting')
    expect(communicationDomainElementsCss).toContain('.mail-sync-progress-region')
    expect(communicationDomainElementsCss).toContain('mail-sync-progress-sheen')
    expect(communicationDomainElementsCss).toContain('mail-sync-progress-breathe')
    expect(communicationDomainElementsCss).toContain('mail-sync-progress-bar-flow')
    expect(communicationDomainElementsCss).toMatch(
      /\.mail-sync-progress__badge\s*\{[^}]*text-transform: none;/s
    )
    expect(communicationDomainElementsCss).toMatch(
      /\.communications-workspace-view\s*\{[^}]*box-sizing: border-box;[^}]*height: 100%;[^}]*max-height: 100%;/s
    )
    expect(communicationDomainElementsCss).not.toContain('calc(100dvh - 72px)')
    expect(communicationDomainElementsCss).toMatch(
      /\.mail-list-stack\s*\{[^}]*height: 100%;[^}]*max-height: 100%;[^}]*overflow: hidden;/s
    )
    expect(communicationDomainElementsCss).toMatch(
      /\.mail-sync-progress-region\s*\{[^}]*max-height: 0;[^}]*transition:[^}]*max-height/s
    )
    expect(communicationDomainElementsCss).toMatch(
      /\.mail-sync-progress-region--visible\s*\{[^}]*max-height: 180px;/s
    )
    expect(communicationDomainElementsCss).toMatch(
      /@media \(max-width: 1180px\)[\s\S]*\.communication-workspace-shell--mail,\s*\.communication-workspace-shell--mail-inspector-hidden\s*\{[\s\S]*grid-template-rows: minmax\(0, 1fr\);/s
    )
    expect(communicationDomainElementsCss).toMatch(
      /@media \(max-width: 1180px\)[\s\S]*\.communication-workspace-shell--mail > \.communication-workspace-panel--inspector\s*\{[\s\S]*display: none;/s
    )
    expect(communicationDomainElementsCss).toContain('grid-template-rows: 0fr')
    expect(communicationDomainElementsCss).toContain('grid-template-rows: 1fr')
    expect(communicationDomainElementsCss).toContain('mail-sync-progress-exit-down')
    expect(viewSurfaceSource).not.toContain('menuItems')
    expect(viewSurfaceSource).not.toContain('routeToMenuItemId')
  })
})
