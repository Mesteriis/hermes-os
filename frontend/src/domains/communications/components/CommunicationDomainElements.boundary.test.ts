import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'
import {
  mailListAccountOptions,
  mailListAllAccountsOptionId,
  mailListItemsForAccount,
  type MailListItemModel,
} from './mail/mailElements'
import {
  mailFolderAriaLabel,
  mailFolderExpandableIds,
  mailFolderExpandedIds,
  mailFolderPresentation,
  mailFolderRows,
  mailStandardFolders,
  type MailFolderModel,
} from './mail/mailFolders'
import {
  createMailListSearchBuilderState,
  mailListItemsForSearch,
  mailListSearchBuilderAddClause,
  mailListSearchBuilderClauseViews,
  mailListSearchBuilderCommittedClauseViews,
  mailListSearchBuilderDraftTokens,
  mailListSearchBuilderQuery,
  mailListSearchBuilderSetField,
  mailListSearchBuilderSetMatchMode,
  mailListSearchBuilderSetOperator,
  mailListSearchBuilderSetValue,
} from './mail/mailSearchBuilder'
import { mailListSearchBuilderValueSuggestions } from './mail/mailSearchSuggestions'

const mailSearchItems: readonly MailListItemModel[] = [
  {
    id: 'provider-record-001',
    accountLabel: 'Work',
    mailboxLabel: 'Inbox',
    fromName: 'Maya Chen',
    fromAddress: 'maya@example.test',
    subject: 'Vendor security review',
    snippet: 'Retention clause is still open.',
    timestampLabel: '09:42',
    workflowState: 'needs_action',
    providerRecordId: 'gmail-msg-security-review-001',
    recipients: ['owner@example.test', 'legal@example.test'],
    labels: ['security', 'work'],
    localState: 'active',
    deliveryState: 'received',
    aiCategory: 'risk',
    importanceScore: 88,
    attachmentCount: 2,
    hermesEntities: [
      { kind: 'organization', title: 'Northwind Security Vendor' },
      { kind: 'decision', title: 'Retention clause approval' },
      { kind: 'document', title: 'Security answers' },
    ],
    evidenceKinds: ['mail_thread', 'attachment', 'redline'],
    taskCandidateCount: 1,
    decisionCandidateCount: 1,
    documentCandidateCount: 2,
    deadlineCount: 1,
    riskCount: 2,
  },
  {
    id: 'provider-record-002',
    accountLabel: 'Work',
    mailboxLabel: 'Inbox',
    fromName: 'Finance Team',
    fromAddress: 'finance@example.test',
    subject: 'Board pack edits',
    snippet: 'Disclosure note was approved.',
    timestampLabel: 'Yesterday',
    workflowState: 'waiting',
  },
]

describe('Communication domain elements', () => {
  it('defines standard provider-neutral mail folders', () => {
    expect(mailStandardFolders.map((folder) => folder.kind)).toEqual([
      'inbox',
      'sent',
      'drafts',
      'outbox',
      'archive',
      'spam',
      'trash',
      'all',
    ])
    const inboxFolder = mailStandardFolders.find(
      (folder) => folder.kind === 'inbox'
    )
    const spamFolder = mailStandardFolders.find(
      (folder) => folder.kind === 'spam'
    )
    expect(inboxFolder).toBeDefined()
    expect(spamFolder).toBeDefined()
    expect(mailFolderPresentation(inboxFolder!).icon).toBe('tabler:inbox')
    expect(mailFolderPresentation(spamFolder!).tone).toBe('danger')
    expect(
      mailFolderAriaLabel({ ...inboxFolder!, count: 42, unreadCount: 7 })
    ).toBe('Inbox, 7 unread, 42 total')
    const hierarchicalFolders = [
      {
        ...inboxFolder!,
        children: [
          { id: 'inbox-work', kind: 'custom', label: 'Work' },
          { id: 'inbox-finance', kind: 'custom', label: 'Finance' },
        ],
      },
    ] satisfies readonly MailFolderModel[]
    expect(mailFolderExpandableIds(hierarchicalFolders)).toEqual(['inbox'])
    expect(mailFolderExpandedIds([], 'inbox', true)).toEqual(['inbox'])
    expect(mailFolderExpandedIds(['inbox'], 'inbox', false)).toEqual([])
    expect(mailFolderRows(hierarchicalFolders, ['inbox'])).toEqual([
      {
        folder: expect.objectContaining({ id: 'inbox' }),
        depth: 1,
        hasChildren: true,
        expanded: true,
      },
      {
        folder: expect.objectContaining({ id: 'inbox-work' }),
        depth: 2,
        hasChildren: false,
        expanded: false,
      },
      {
        folder: expect.objectContaining({ id: 'inbox-finance' }),
        depth: 2,
        hasChildren: false,
        expanded: false,
      },
    ])
    expect(mailFolderRows(hierarchicalFolders, [])).toEqual([
      {
        folder: expect.objectContaining({ id: 'inbox' }),
        depth: 1,
        hasChildren: true,
        expanded: false,
      },
    ])
  })

  it('matches backend q fields plus Communications read-model facets in the mail list helper', () => {
    expect(mailListItemsForSearch(mailSearchItems, 'from:Maya')).toHaveLength(1)
    expect(
      mailListItemsForSearch(mailSearchItems, 'subject:board body:approved')
    ).toHaveLength(1)
    expect(
      mailListItemsForSearch(
        mailSearchItems,
        'mode:any subject:missing body:retention'
      )
    ).toHaveLength(1)
    expect(
      mailListItemsForSearch(mailSearchItems, 'all=provider-record-001')
    ).toHaveLength(1)
    expect(
      mailListItemsForSearch(mailSearchItems, 'label:security')
    ).toHaveLength(1)
    expect(
      mailListItemsForSearch(
        mailSearchItems,
        'attachment:yes workflow=needs_action'
      )
    ).toHaveLength(1)
    expect(
      mailListItemsForSearch(mailSearchItems, 'entity:organization task:yes')
    ).toHaveLength(1)
    expect(
      mailListItemsForSearch(mailSearchItems, 'importance>=75')
    ).toHaveLength(1)
  })

  it('builds structured mail search clauses into the backend q syntax', () => {
    let builder = createMailListSearchBuilderState()
    builder = mailListSearchBuilderSetValue(builder, 'Maya Chen')
    builder = mailListSearchBuilderAddClause(builder)
    builder = mailListSearchBuilderSetMatchMode(builder, 'any')
    builder = mailListSearchBuilderSetField(builder, 'body')
    builder = mailListSearchBuilderSetValue(builder, 'retention')

    expect(mailListSearchBuilderQuery(builder)).toBe(
      'mode:any from:"Maya Chen" body:retention'
    )
    expect(mailListSearchBuilderClauseViews(builder)).toHaveLength(2)
    expect(mailListSearchBuilderCommittedClauseViews(builder)).toHaveLength(1)
    expect(
      mailListSearchBuilderDraftTokens(builder).map((token) => token.value)
    ).toEqual(['body', 'contains'])
    expect(
      mailListItemsForSearch(
        mailSearchItems,
        mailListSearchBuilderQuery(builder)
      )
    ).toHaveLength(1)
  })

  it('builds rich Communications and Hermes search clauses', () => {
    let builder = createMailListSearchBuilderState()
    builder = mailListSearchBuilderSetField(builder, 'entity')
    builder = mailListSearchBuilderSetValue(builder, 'organization')
    builder = mailListSearchBuilderAddClause(builder)
    builder = mailListSearchBuilderSetField(builder, 'attachment')
    builder = mailListSearchBuilderSetValue(builder, 'yes')
    builder = mailListSearchBuilderAddClause(builder)
    builder = mailListSearchBuilderSetField(builder, 'importance')
    builder = mailListSearchBuilderSetOperator(builder, 'gte')
    builder = mailListSearchBuilderSetValue(builder, '75')

    expect(mailListSearchBuilderQuery(builder)).toBe(
      'entity:organization attachment:yes importance>=75'
    )
    expect(
      mailListItemsForSearch(
        mailSearchItems,
        mailListSearchBuilderQuery(builder)
      )
    ).toHaveLength(1)
  })

  it('suggests lazy search values from mail attributes and Hermes entities', () => {
    let builder = createMailListSearchBuilderState()
    builder = mailListSearchBuilderSetValue(builder, 'may')

    expect(
      mailListSearchBuilderValueSuggestions(mailSearchItems, builder)
    ).toContainEqual({
      value: 'Maya Chen',
      label: 'Maya Chen',
    })

    builder = mailListSearchBuilderSetField(builder, 'label')
    builder = mailListSearchBuilderSetValue(builder, 'sec')
    expect(
      mailListSearchBuilderValueSuggestions(mailSearchItems, builder)
    ).toContainEqual({
      value: 'security',
      label: 'security',
    })

    builder = mailListSearchBuilderSetField(builder, 'entity')
    expect(
      mailListSearchBuilderValueSuggestions(mailSearchItems, builder)
    ).toEqual(
      expect.arrayContaining([
        { value: 'organization', label: 'organization' },
        {
          value: 'Northwind Security Vendor',
          label: 'Northwind Security Vendor',
        },
      ])
    )

    builder = mailListSearchBuilderSetField(builder, 'attachment')
    expect(
      mailListSearchBuilderValueSuggestions(mailSearchItems, builder)
    ).toEqual([
      { value: 'yes', label: 'yes' },
      { value: 'no', label: 'no' },
    ])
  })

  it('keeps mail account filtering separate from the search q syntax', () => {
    const accountOptions = mailListAccountOptions(mailSearchItems)
    const workAccount = accountOptions.find((option) => option.label === 'Work')

    expect(accountOptions[0]).toEqual({
      id: mailListAllAccountsOptionId,
      label: 'All accounts',
      count: 2,
    })
    expect(workAccount).toBeDefined()
    expect(
      mailListItemsForAccount(
        mailSearchItems,
        workAccount?.id ?? mailListAllAccountsOptionId
      )
    ).toHaveLength(2)
    expect(
      mailListItemsForAccount(mailSearchItems, mailListAllAccountsOptionId)
    ).toHaveLength(2)
  })

  it('adds provider-neutral domain elements without reintroducing provider product panels', () => {
    const helperSource = readFileSync(
      new URL('./communicationDomainElements.ts', import.meta.url),
      'utf8'
    )
    const domainCssSource = readFileSync(
      new URL('./communicationDomainElements.css', import.meta.url),
      'utf8'
    )
    const storybookCssSource = readFileSync(
      new URL('../../../shared/ui/styles/storybook.css', import.meta.url),
      'utf8'
    )
    const shellSource = readFileSync(
      new URL('./CommunicationWorkspaceShell.vue', import.meta.url),
      'utf8'
    )
    const inboxSource = readFileSync(
      new URL('./CommunicationInboxList.vue', import.meta.url),
      'utf8'
    )
    const conversationSource = readFileSync(
      new URL('./CommunicationConversationPane.vue', import.meta.url),
      'utf8'
    )
    const inspectorSource = readFileSync(
      new URL('./CommunicationHermesInspector.vue', import.meta.url),
      'utf8'
    )
    const mailFolderListSource = readFileSync(
      new URL('./mail/MailFolderList.vue', import.meta.url),
      'utf8'
    )
    const mailFoldersSource = readFileSync(
      new URL('./mail/mailFolders.ts', import.meta.url),
      'utf8'
    )
    const mailListSource = readFileSync(
      new URL('./mail/MailList.vue', import.meta.url),
      'utf8'
    )
    const mailListItemSource = readFileSync(
      new URL('./mail/MailListItem.vue', import.meta.url),
      'utf8'
    )
    const mailListViewsSource = readFileSync(
      new URL('./mail/mailListViews.ts', import.meta.url),
      'utf8'
    )
    const mailElementsSource = readFileSync(
      new URL('./mail/mailElements.ts', import.meta.url),
      'utf8'
    )
    const mailSearchSource = readFileSync(
      new URL('./mail/mailSearchBuilder.ts', import.meta.url),
      'utf8'
    )
    const mailSearchSuggestionSource = readFileSync(
      new URL('./mail/mailSearchSuggestions.ts', import.meta.url),
      'utf8'
    )
    const mailMessageSource = readFileSync(
      new URL('./mail/MailMessage.vue', import.meta.url),
      'utf8'
    )
    const mailActionSource = readFileSync(
      new URL('./mail/MailAction.vue', import.meta.url),
      'utf8'
    )
    const mailActionsSource = readFileSync(
      new URL('./mail/mailActions.ts', import.meta.url),
      'utf8'
    )
    const mailActionQueriesSource = readFileSync(
      new URL('../queries/mailActionQueries.ts', import.meta.url),
      'utf8'
    )
    const mailOperationQueriesSource = readFileSync(
      new URL('../queries/mailOperationQueries.ts', import.meta.url),
      'utf8'
    )
    const messageApiSource = readFileSync(
      new URL('../api/messageApi.ts', import.meta.url),
      'utf8'
    )
    const mailViewerSource = readFileSync(
      new URL('./mail/MailViewer.vue', import.meta.url),
      'utf8'
    )
    const mailFooterSource = readFileSync(
      new URL('./mail/MailFooter.vue', import.meta.url),
      'utf8'
    )
    const mailInspectorSource = readFileSync(
      new URL('./mail/MailInspector.vue', import.meta.url),
      'utf8'
    )
    const mailInspectorModelSource = readFileSync(
      new URL('./mail/mailInspector.ts', import.meta.url),
      'utf8'
    )
    const messengerListSource = readFileSync(
      new URL('./messengers/MessengerList.vue', import.meta.url),
      'utf8'
    )
    const messengerListItemSource = readFileSync(
      new URL('./messengers/MessengerListItem.vue', import.meta.url),
      'utf8'
    )
    const messengerActionSource = readFileSync(
      new URL('./messengers/MessengerAction.vue', import.meta.url),
      'utf8'
    )
    const messengerViewerSource = readFileSync(
      new URL('./messengers/MessengerViewer.vue', import.meta.url),
      'utf8'
    )
    const messengerRichEditorSource = readFileSync(
      new URL('./messengers/MessengerRichEditor.vue', import.meta.url),
      'utf8'
    )
    const messengerProviderRichEditorSource = readFileSync(
      new URL('./messengers/MessengerProviderRichEditor.vue', import.meta.url),
      'utf8'
    )
    const telegramMessengerRichEditorSource = readFileSync(
      new URL('./messengers/TelegramMessengerRichEditor.vue', import.meta.url),
      'utf8'
    )
    const whatsAppMessengerRichEditorSource = readFileSync(
      new URL('./messengers/WhatsAppMessengerRichEditor.vue', import.meta.url),
      'utf8'
    )
    const signalMessengerRichEditorSource = readFileSync(
      new URL('./messengers/SignalMessengerRichEditor.vue', import.meta.url),
      'utf8'
    )
    const messengerMessageSource = readFileSync(
      new URL('./messengers/MessengerMessage.vue', import.meta.url),
      'utf8'
    )
    const messengerInspectorSource = readFileSync(
      new URL('./messengers/MessengerInspector.vue', import.meta.url),
      'utf8'
    )
    const messengerWorkspaceSource = readFileSync(
      new URL('./messengers/MessengerWorkspace.vue', import.meta.url),
      'utf8'
    )
    const messengerElementsSource = readFileSync(
      new URL('./messengers/messengerElements.ts', import.meta.url),
      'utf8'
    )
    const messengerComposerSource = readFileSync(
      new URL('./messengers/messengerComposer.ts', import.meta.url),
      'utf8'
    )
    const mailQuotedOriginalSource = readFileSync(
      new URL('./mail/MailQuotedOriginal.vue', import.meta.url),
      'utf8'
    )
    const mailReplyComposerSource = readFileSync(
      new URL('./mail/MailReplyComposer.vue', import.meta.url),
      'utf8'
    )
    const mailWorkspaceSource = readFileSync(
      new URL('./mail/MailWorkspace.vue', import.meta.url),
      'utf8'
    )
    const channelWorkspaceSource = readFileSync(
      new URL('./CommunicationChannelWorkspace.vue', import.meta.url),
      'utf8'
    )
    const channelListSource = readFileSync(
      new URL('./channels/ChannelList.vue', import.meta.url),
      'utf8'
    )
    const channelActionSource = readFileSync(
      new URL('./channels/ChannelAction.vue', import.meta.url),
      'utf8'
    )
    const channelViewerSource = readFileSync(
      new URL('./channels/ChannelViewer.vue', import.meta.url),
      'utf8'
    )
    const channelInspectorSource = readFileSync(
      new URL('./channels/ChannelInspector.vue', import.meta.url),
      'utf8'
    )
    const channelWorkspaceComponentSource = readFileSync(
      new URL('./channels/ChannelWorkspace.vue', import.meta.url),
      'utf8'
    )
    const channelSurfaceAdaptersSource = readFileSync(
      new URL('./channels/channelSurfaceAdapters.ts', import.meta.url),
      'utf8'
    )
    const communicationSurfaceSource = readFileSync(
      new URL('../queries/communicationChannelSurface.ts', import.meta.url),
      'utf8'
    )
    const callsSurfaceSource = readFileSync(
      new URL('./CommunicationCallsSurface.vue', import.meta.url),
      'utf8'
    )
    const callElementsSource = readFileSync(
      new URL('./calls/callElements.ts', import.meta.url),
      'utf8'
    )
    const callListItemSource = readFileSync(
      new URL('./calls/CallListItem.vue', import.meta.url),
      'utf8'
    )
    const callListSource = readFileSync(
      new URL('./calls/CallList.vue', import.meta.url),
      'utf8'
    )
    const callActionSource = readFileSync(
      new URL('./calls/CallAction.vue', import.meta.url),
      'utf8'
    )
    const callViewerSource = readFileSync(
      new URL('./calls/CallViewer.vue', import.meta.url),
      'utf8'
    )
    const callInspectorSource = readFileSync(
      new URL('./calls/CallInspector.vue', import.meta.url),
      'utf8'
    )
    const callMessageSource = readFileSync(
      new URL('./calls/CallMessage.vue', import.meta.url),
      'utf8'
    )
    const callWorkspaceSource = readFileSync(
      new URL('./calls/CallWorkspace.vue', import.meta.url),
      'utf8'
    )
    const outboxSource = readFileSync(
      new URL('./CommunicationOutboxStatusCard.vue', import.meta.url),
      'utf8'
    )
    const primitiveStorySource = readFileSync(
      new URL(
        '../../../../stories/ui/Communication.stories.ts',
        import.meta.url
      ),
      'utf8'
    )
    const mailStorySource = readFileSync(
      new URL(
        '../../../../stories/app/CommunicationMail.stories.ts',
        import.meta.url
      ),
      'utf8'
    )
    const messengerStorySource = readFileSync(
      new URL(
        '../../../../stories/app/CommunicationMessengers.stories.ts',
        import.meta.url
      ),
      'utf8'
    )
    const channelStorySource = readFileSync(
      new URL(
        '../../../../stories/app/CommunicationChannels.stories.ts',
        import.meta.url
      ),
      'utf8'
    )
    const callsStorySource = readFileSync(
      new URL(
        '../../../../stories/app/CommunicationCalls.stories.ts',
        import.meta.url
      ),
      'utf8'
    )
    const storySources = [
      primitiveStorySource,
      mailStorySource,
      messengerStorySource,
      channelStorySource,
      callsStorySource,
    ]

    expect(
      existsSync(new URL('./CommunicationInboxList.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(
        new URL('./CommunicationConversationPane.vue', import.meta.url)
      )
    ).toBe(true)
    expect(
      existsSync(new URL('./CommunicationHermesInspector.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./CommunicationWorkspaceShell.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./mail/MailFolderList.vue', import.meta.url))
    ).toBe(true)
    expect(existsSync(new URL('./mail/mailFolders.ts', import.meta.url))).toBe(
      true
    )
    expect(existsSync(new URL('./mail/MailList.vue', import.meta.url))).toBe(
      true
    )
    expect(
      existsSync(new URL('./mail/MailListItem.vue', import.meta.url))
    ).toBe(true)
    expect(existsSync(new URL('./mail/mailElements.ts', import.meta.url))).toBe(
      true
    )
    expect(
      existsSync(new URL('./mail/mailSearchBuilder.ts', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./mail/mailSearchSuggestions.ts', import.meta.url))
    ).toBe(true)
    expect(existsSync(new URL('./mail/MailMessage.vue', import.meta.url))).toBe(
      true
    )
    expect(existsSync(new URL('./mail/MailAction.vue', import.meta.url))).toBe(
      true
    )
    expect(existsSync(new URL('./mail/mailActions.ts', import.meta.url))).toBe(
      true
    )
    expect(
      existsSync(new URL('../queries/mailActionQueries.ts', import.meta.url))
    ).toBe(true)
    expect(existsSync(new URL('./mail/MailViewer.vue', import.meta.url))).toBe(
      true
    )
    expect(existsSync(new URL('./mail/MailFooter.vue', import.meta.url))).toBe(
      true
    )
    expect(
      existsSync(new URL('./mail/MailInspector.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./mail/mailInspector.ts', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./messengers/MessengerList.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./messengers/MessengerListItem.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./messengers/MessengerAction.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./messengers/MessengerViewer.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(
        new URL('./messengers/MessengerRichEditor.vue', import.meta.url)
      )
    ).toBe(true)
    expect(
      existsSync(
        new URL('./messengers/MessengerProviderRichEditor.vue', import.meta.url)
      )
    ).toBe(true)
    expect(
      existsSync(
        new URL('./messengers/TelegramMessengerRichEditor.vue', import.meta.url)
      )
    ).toBe(true)
    expect(
      existsSync(
        new URL('./messengers/WhatsAppMessengerRichEditor.vue', import.meta.url)
      )
    ).toBe(true)
    expect(
      existsSync(
        new URL('./messengers/SignalMessengerRichEditor.vue', import.meta.url)
      )
    ).toBe(true)
    expect(
      existsSync(new URL('./messengers/MessengerMessage.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(
        new URL('./messengers/MessengerInspector.vue', import.meta.url)
      )
    ).toBe(true)
    expect(
      existsSync(
        new URL('./messengers/MessengerWorkspace.vue', import.meta.url)
      )
    ).toBe(true)
    expect(
      existsSync(new URL('./messengers/messengerElements.ts', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./messengers/messengerComposer.ts', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./mail/MailQuotedOriginal.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./mail/MailReplyComposer.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./mail/MailWorkspace.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(
        new URL('./CommunicationChannelWorkspace.vue', import.meta.url)
      )
    ).toBe(true)
    expect(
      existsSync(new URL('./channels/ChannelListItem.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./channels/ChannelList.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./channels/ChannelAction.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(
        new URL('./channels/channelSurfaceAdapters.ts', import.meta.url)
      )
    ).toBe(true)
    expect(
      existsSync(new URL('./channels/ChannelViewer.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./channels/ChannelInspector.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./channels/ChannelMessage.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./channels/ChannelWorkspace.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./CommunicationCallsSurface.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./calls/CallListItem.vue', import.meta.url))
    ).toBe(true)
    expect(existsSync(new URL('./calls/CallList.vue', import.meta.url))).toBe(
      true
    )
    expect(existsSync(new URL('./calls/CallAction.vue', import.meta.url))).toBe(
      true
    )
    expect(existsSync(new URL('./calls/CallViewer.vue', import.meta.url))).toBe(
      true
    )
    expect(
      existsSync(new URL('./calls/CallInspector.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./calls/CallMessage.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(new URL('./calls/CallWorkspace.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(
        new URL('./CommunicationChannelSurfaceCard.vue', import.meta.url)
      )
    ).toBe(true)
    expect(
      existsSync(new URL('./CommunicationCapabilityCard.vue', import.meta.url))
    ).toBe(true)
    expect(
      existsSync(
        new URL('./CommunicationThreadSignalCard.vue', import.meta.url)
      )
    ).toBe(true)
    expect(
      existsSync(
        new URL('./CommunicationOutboxStatusCard.vue', import.meta.url)
      )
    ).toBe(true)
    expect(
      existsSync(
        new URL('./CommunicationWorkspaceOverview.vue', import.meta.url)
      )
    ).toBe(true)

    expect(helperSource).toContain('CommunicationThreadSummary')
    expect(helperSource).toContain('CommunicationOutboxItem')
    expect(helperSource).toContain('CommunicationInboxItemModel')
    expect(helperSource).toContain('CommunicationConversationModel')
    expect(helperSource).toContain('CommunicationMessageAttributeGroupModel')
    expect(helperSource).toContain('CommunicationMessageActionGroupModel')
    expect(helperSource).toContain("bodyFormat?: 'plain' | 'html'")
    expect(helperSource).toContain('bodyHtmlSanitized?: boolean')
    expect(helperSource).toContain('CommunicationHermesInspectorSectionModel')
    expect(helperSource).toContain('CommunicationChannelWorkspaceModel')
    expect(helperSource).toContain('CommunicationChannelProviderKind')
    expect(helperSource).toContain('CommunicationChannelActionModel')
    expect(helperSource).toContain('CommunicationChannelActionGroupModel')
    expect(helperSource).toContain(
      'actionGroups: readonly CommunicationChannelActionGroupModel[]'
    )
    expect(helperSource).toContain('CommunicationChannelInspectorModel')
    expect(helperSource).toContain('CommunicationChannelInspectorIntelligence')
    expect(helperSource).toContain('CommunicationChannelInspectorActionItem')
    expect(helperSource).toContain(
      'inspector: CommunicationChannelInspectorModel'
    )
    expect(helperSource).toContain('CommunicationChannelDirectChatModel')
    expect(helperSource).toContain('CommunicationChannelDirectFolderModel')
    expect(helperSource).toContain(
      'directChatFolders: readonly CommunicationChannelDirectFolderModel[]'
    )
    expect(helperSource).toContain('communicationChannelDirectChatCount')
    expect(helperSource).toContain('communicationChannelProviderIconName')
    expect(helperSource).toContain("from './calls/callElements'")
    expect(callElementsSource).toContain('CommunicationCallsSurfaceModel')
    expect(callElementsSource).toContain('CommunicationCallProviderKind')
    expect(callElementsSource).toContain("'zoom'")
    expect(callElementsSource).toContain("'telemost'")
    expect(callElementsSource).toContain("'zulip'")
    expect(callElementsSource).toContain('CommunicationPermanentCallLinkModel')
    expect(callElementsSource).toContain(
      'permanentMeetings: readonly CommunicationPermanentCallLinkModel[]'
    )
    expect(callElementsSource).toContain('dateGroupLabel: string')
    expect(callElementsSource).toContain('sortKey: string')
    expect(callElementsSource).toContain('CommunicationCallActionGroupModel')
    expect(callElementsSource).toContain('CommunicationCallInspectorModel')
    expect(callElementsSource).toContain('communicationCallProviderLabel')
    expect(helperSource).toContain('outboxStatusPresentation')
    expect(helperSource).toContain("channelKind === 'telegram'")
    expect(helperSource).toContain("channelKind === 'whatsapp'")
    expect(helperSource).toContain(
      "channelKind === 'mail' || channelKind === 'email'"
    )
    expect(helperSource).toContain('communicationConversationIsEmail')

    expect(shellSource).toContain('CommunicationInboxList')
    expect(shellSource).toContain('CommunicationConversationPane')
    expect(shellSource).toContain('CommunicationHermesInspector')
    expect(inboxSource).toContain('Chats, groups and mail threads')
    expect(mailListSource).toContain('MailListItem')
    expect(mailListSource).toContain(
      "import { useI18n } from '@/platform/i18n'"
    )
    expect(mailListSource).not.toContain('mail-list-account-select')
    expect(mailListSource).not.toContain("t('Mail account')")
    expect(mailListSource).not.toContain('Email threads with attachments')
    expect(mailListSource).not.toContain(
      'Build the same query with structured tokens.'
    )
    expect(mailListSource).not.toContain('Mail structured search')
    expect(mailListSource).not.toContain('mail-list-toolbar')
    expect(mailListSource).toContain('Popover')
    expect(mailListSource).toContain('hermes-icon-button')
    expect(mailListSource).toContain("t('Search builder')")
    expect(mailListSource).toContain('TreeSelect')
    expect(mailListSource).toContain('savedFilterName')
    expect(mailListSource).toContain('mail-search-builder__token')
    expect(mailListSource).toContain('searchBuilderFieldGroups')
    expect(mailListSource).toContain('activeSearchBuilderGroupId')
    expect(mailListSource).toContain('mail-search-builder__group-tab')
    expect(mailListSource).toContain('mail-search-builder__rule-panel')
    expect(mailListSource).toContain('mail-search-builder__value-row')
    expect(mailListSource).toContain('mail-search-builder__field-option')
    expect(mailListSource).toContain('searchBuilderPresetOptions')
    expect(mailListSource).toContain('searchBuilderValueSuggestions')
    expect(mailListSource).toContain('Combobox')
    expect(mailListSource).toContain('builderSearchQuery')
    expect(mailListSource).toContain('ToggleGroup')
    expect(mailListSource).toContain('visibleItems')
    expect(mailListSource).toContain('mailListDensityToggleItems')
    expect(mailListSource).toContain('DropdownMenu')
    expect(mailListSource).toContain('mail-list-settings-menu')
    expect(mailListSource).toContain("t('List display')")
    expect(mailListSource).toContain('mail-list-plain-search')
    expect(mailListSource).toContain("'update-search-query'")
    expect(mailListSource).toContain("t('Address, subject or body')")
    expect(mailListSource).toContain('plainSearchQuery || builderSearchQuery')
    expect(mailListSource).toContain('activeDensity')
    expect(mailListSource).toContain('mailViewOptions')
    expect(mailListSource).toContain('activeMailViewId')
    expect(mailListSource).not.toContain('.slice(0, 24)')
    expect(mailListViewsSource).toContain("'mail:all'")
    expect(mailListViewsSource).toContain("'mail:spam'")
    expect(mailListViewsSource).toContain("'mail:drafts'")
    expect(mailListViewsSource).toContain("item.mailboxLabel === 'Inbox'")
    expect(mailListViewsSource).toContain("item.mailboxLabel === 'Sent'")
    expect(mailListViewsSource).toContain("item.mailboxLabel === 'Trash'")
    expect(mailListViewsSource).toContain('mailListItemsForView')
    expect(mailListViewsSource).toContain('mailListTreeSelectOptions')
    expect(mailListViewsSource).toContain("translate('Saved filters')")
    expect(mailFolderListSource).toContain('MailFolderModel')
    expect(mailFolderListSource).toContain("t('Mail folders')")
    expect(mailFolderListSource).toContain('role="tree"')
    expect(mailFolderListSource).toContain('role="treeitem"')
    expect(mailFolderListSource).toContain(':aria-level="row.depth"')
    expect(mailFolderListSource).toContain(
      ':aria-expanded="row.hasChildren ? row.expanded : undefined"'
    )
    expect(mailFolderListSource).toContain('expandedFolderIds')
    expect(mailFolderListSource).toContain('toggleFolder')
    expect(mailFolderListSource).toContain('mailFolderToggleAriaLabel')
    expect(mailFolderListSource).toContain('mailFolderLocalizedAriaLabel')
    expect(mailFolderListSource).toContain('mail-folder-list__item--active')
    expect(mailFolderListSource).toContain('mailFolderDepthClass')
    expect(mailFolderListSource).toContain('mailFolderPresentation')
    expect(mailFoldersSource).toContain('mailStandardFolders')
    expect(mailFoldersSource).toContain('children?: readonly MailFolderModel[]')
    expect(mailFoldersSource).toContain('mailFolderExpandableIds')
    expect(mailFoldersSource).toContain('mailFolderExpandedIds')
    expect(mailFoldersSource).toContain('mailFolderRows')
    expect(mailFoldersSource).toContain("'custom'")
    expect(mailFoldersSource).toContain("'inbox'")
    expect(mailFoldersSource).toContain("'sent'")
    expect(mailFoldersSource).toContain("'drafts'")
    expect(mailFoldersSource).toContain("'outbox'")
    expect(mailFoldersSource).toContain("'archive'")
    expect(mailFoldersSource).toContain("'spam'")
    expect(mailFoldersSource).toContain("'trash'")
    expect(mailFoldersSource).toContain("'all'")
    expect(mailListItemSource).toContain('MailListItemModel')
    expect(mailListItemSource).toContain('fromName')
    expect(mailListItemSource).toContain('subject')
    expect(mailListItemSource).toContain('snippet')
    expect(mailListItemSource).toContain('Tooltip')
    expect(mailListItemSource).toContain('mail-list-item__primary')
    expect(mailListItemSource).toContain('mail-list-item__summary')
    expect(mailListItemSource).toContain('mail-list-item__signals')
    expect(mailListItemSource).toContain('mail-list-item__status-dot')
    expect(mailListItemSource).toContain('mailListItemStatusChipClass')
    expect(mailListItemSource).toContain('density?: MailListItemDensity')
    expect(mailListItemSource).not.toContain('timelineHint')
    expect(mailListItemSource).not.toContain('mailListItemConfidenceClass')
    expect(mailListItemSource).toContain('mailListItemMarkerClass')
    expect(mailElementsSource).toContain('MailListItemModel')
    expect(mailElementsSource).toContain('MailListItemConfidence')
    expect(mailElementsSource).toContain('MailListItemCounter')
    expect(mailElementsSource).toContain('MailListItemDensity')
    expect(mailElementsSource).toContain('MailListAccountOption')
    expect(mailElementsSource).toContain('mailListAccountOptions')
    expect(mailElementsSource).toContain('mailListItemsForAccount')
    expect(mailElementsSource).toContain('mailListAllAccountsOptionId')
    expect(mailElementsSource).toContain('mailListItemStatus')
    expect(mailElementsSource).toContain('mailListItemHasSignal')
    expect(mailElementsSource).toContain('mailListItemAttachmentLabel')
    expect(mailElementsSource).toContain('mailListItemAriaLabel')
    expect(mailElementsSource).toContain('mailListItemCounters')
    expect(mailElementsSource).toContain('mailListItemSourceKind')
    expect(mailElementsSource).toContain('MailListItemMarker')
    expect(mailElementsSource).toContain('mailListItemMarkerPresentation')
    expect(mailElementsSource).toContain('mailListItemMarkerClass')
    expect(mailElementsSource).toContain('mailListItemMarkerSummary')
    expect(mailElementsSource).toContain('mailListItemAiIndicator')
    expect(mailElementsSource).toContain('AI processing is available')
    expect(mailElementsSource).toContain('mailListItemDensityOptions')
    expect(mailElementsSource).toContain('mailListDensityToggleItems')
    expect(mailElementsSource).toContain('iconOnly: true')
    expect(mailElementsSource).toContain("icon: 'tabler:list'")
    expect(mailElementsSource).toContain("icon: 'tabler:list-details'")
    expect(mailElementsSource).toContain("icon: 'tabler:layout-list'")
    expect(mailElementsSource).toContain('providerRecordId')
    expect(mailElementsSource).toContain('recipients')
    expect(mailElementsSource).toContain('hermesEntities')
    expect(mailElementsSource).toContain('evidenceKinds')
    expect(mailElementsSource).toContain('importanceScore')
    expect(mailElementsSource).toContain('aiState')
    expect(mailSearchSource).toContain('mailListSearchFieldGroups')
    expect(mailSearchSource).toContain('Mail attrs')
    expect(mailSearchSource).toContain('Hermes')
    expect(mailSearchSource).toContain("'workflow'")
    expect(mailSearchSource).toContain("'attachment'")
    expect(mailSearchSource).toContain("'entity'")
    expect(mailSearchSource).toContain("'importance'")
    expect(mailSearchSource).toContain("'ai_category'")
    expect(mailSearchSource).toContain("'task'")
    expect(mailSearchSource).toContain('mode:(all|any)')
    expect(mailSearchSuggestionSource).toContain(
      'mailListSearchBuilderValueSuggestions'
    )
    expect(mailSearchSuggestionSource).toContain(
      'mailListEntitySuggestionValues'
    )
    expect(mailSearchSuggestionSource).toContain('booleanSuggestionValues')
    expect(mailElementsSource).toContain('tabler:mail-x')
    expect(mailElementsSource).toContain('tabler:sparkles')
    expect(mailElementsSource).toContain('tabler:shield-exclamation')
    expect(mailElementsSource).toContain("'spam'")
    expect(mailElementsSource).toContain("'ai-processed'")
    expect(mailElementsSource).toContain("'phishing'")
    expect(mailElementsSource).toContain("label: 'AI'")
    expect(mailElementsSource).not.toContain('CommunicationInboxItemModel')
    expect(mailMessageSource).toContain('MailAction')
    expect(mailMessageSource).toContain('MailViewer')
    expect(mailMessageSource).toContain('MailFooter')
    expect(mailMessageSource).toContain('communication-email-preview')
    expect(mailMessageSource).not.toContain('communication-email-center')
    expect(mailMessageSource).not.toContain('HtmlPreview')
    expect(mailActionSource).toContain('communication-email-command-bar')
    expect(mailActionSource).toContain('actionGroups')
    expect(mailActionSource).toContain('ButtonGroup')
    expect(mailActionSource).toContain('Spacer')
    expect(mailActionSource).toContain('SplitButton')
    expect(mailActionSource).toContain('responseControls')
    expect(mailActionSource).toContain('ToolbarGroup')
    expect(mailActionSource).toContain('mailActionResponseControls')
    expect(mailActionSource).toContain('mailActionToolbarSections')
    expect(mailActionSource).toContain('communication-email-action-split')
    expect(mailActionSource).toContain('communication-email-response-group')
    expect(mailActionSource).toContain(
      'communication-email-command-bar__spacer'
    )
    expect(mailActionSource).toContain(
      'communication-email-command-bar__inspector-toggle'
    )
    expect(mailActionSource).toContain(':aria-label="control.label"')
    expect(mailActionSource).not.toContain('{{ control.label }}')
    expect(domainCssSource).toContain('clip-path: inset(50%)')
    expect(mailActionSource).toContain('toggle-inspector')
    expect(mailActionSource).toContain('inspectorVisible')
    expect(mailActionSource).toContain('showInspectorToggle')
    expect(mailActionsSource).toContain('mailActionMenuGroups')
    expect(mailActionsSource).toContain('mailActionResponseControls')
    expect(mailActionsSource).toContain('mailActionToolbarSections')
    expect(mailActionsSource).toContain(
      "itemIds: ['ai-reply', 'ai-reply-variants', 'bilingual-reply-flow', 'smart-cc']"
    )
    expect(mailActionsSource).toContain("itemIds: ['forward-eml', 'redirect']")
    expect(mailActionSource).not.toContain('Dialog')
    expect(mailActionSource).not.toContain('IconButton')
    expect(mailActionSource).not.toContain('ToggleGroup')
    expect(mailActionSource).not.toContain('Configure action panel')
    expect(mailActionSource).not.toContain('Action panel settings')
    expect(mailActionSource).not.toContain('selectedActionIds')
    expect(mailActionSource).not.toContain('action.label')
    expect(domainCssSource).not.toContain(
      'communication-email-command-bar__settings'
    )
    expect(domainCssSource).not.toContain(
      'communication-email-action-settings-dialog'
    )
    expect(mailActionSource).not.toContain('Tooltip')
    expect(mailActionsSource).toContain('Create from message')
    expect(mailActionsSource).toContain('Forwarding actions')
    expect(mailActionsSource).toContain('reply-all')
    expect(mailActionsSource).toContain('forward-eml')
    expect(mailActionsSource).toContain('redirect')
    expect(mailActionsSource).toContain('ai-reply-variants')
    expect(mailActionsSource).toContain('bilingual-reply-flow')
    expect(mailActionsSource).toContain('mark-unread')
    expect(mailActionsSource).toContain('restore-trash')
    expect(mailActionsSource).toContain('bulk-actions')
    expect(mailActionsSource).toContain('remove-label')
    expect(mailActionsSource).toContain('update-ai-state')
    expect(mailActionsSource).toContain('spf-dkim')
    expect(mailActionsSource).toContain('export-md')
    expect(mailActionsSource).toContain('Danger zone')
    expect(mailActionsSource).toContain('mark-spam')
    expect(mailActionsSource).toContain('mark-not-spam')
    expect(mailActionsSource).toContain('delete-provider')
    expect(mailActionQueriesSource).toContain('useMarkMessageSpamMutation')
    expect(mailActionQueriesSource).toContain('useMarkMessageUnreadMutation')
    expect(mailActionQueriesSource).toContain('useRemoveMessageLabelMutation')
    expect(mailOperationQueriesSource).toContain(
      'useUpdateMessageAiStateMutation'
    )
    expect(mailOperationQueriesSource).toContain('useBulkMessageActionMutation')
    expect(messageApiSource).toContain('restoreMessage')
    expect(mailActionQueriesSource).toContain(
      "transitionMessageWorkflowState(messageId, 'spam')"
    )
    expect(mailViewerSource).toContain('MailQuotedOriginal')
    expect(mailViewerSource).toContain('HtmlPreview')
    expect(mailViewerSource).toContain('communication-email-viewer')
    expect(mailViewerSource).toContain(
      'communication-email-viewer__mode-divider'
    )
    expect(mailViewerSource).toContain(
      'communication-email-center__body-scroll'
    )
    expect(mailViewerSource).toContain('communication-email-center__paper')
    expect(domainCssSource).toContain(
      '.communication-email-center__paper {\n\talign-content: start;\n\tgap: var(--h-spacing-3);\n\tbox-sizing: border-box;\n\twidth: 100%;'
    )
    expect(domainCssSource).not.toContain('width: min(820px, 100%);')
    expect(domainCssSource).toContain(
      '.communication-email-message__body-preview .hermes-html-preview__content > :where(table, div, section, article)'
    )
    expect(mailViewerSource).not.toContain('AttachmentChip')
    expect(mailViewerSource).not.toContain(
      'communication-email-attachment-dock'
    )
    expect(mailFooterSource).toContain('communication-email-footer')
    expect(mailFooterSource).toContain('AttachmentChip')
    expect(mailInspectorSource).toContain('MailInspectorModel')
    expect(mailInspectorSource).toContain("t('Email Intelligence')")
    expect(mailInspectorSource).toContain("t('Extracted entities')")
    expect(mailInspectorSource).toContain("t('Suggested actions')")
    expect(mailInspectorSource).toContain("t('Related context')")
    expect(mailInspectorSource).toContain('ScoreGauge')
    expect(mailInspectorSource).toContain('EntityIcon')
    expect(mailInspectorSource).toContain('model.intelligence.checks')
    expect(mailInspectorSource).toContain('model.entityGroups')
    expect(mailInspectorSource).toContain('model.suggestedActions')
    expect(mailInspectorSource).toContain('model.relatedContext')
    expect(mailInspectorModelSource).toContain('MailInspectorModel')
    expect(mailInspectorModelSource).toContain('MailInspectorIntelligence')
    expect(mailInspectorModelSource).toContain('MailInspectorActionItem')
    expect(mailInspectorModelSource).toContain('MailInspectorContextItem')
    expect(messengerElementsSource).toContain('MessengerListItemModel')
    expect(messengerElementsSource).toContain('MessengerConversationModel')
    expect(messengerElementsSource).toContain('MessengerInspectorModel')
    expect(messengerElementsSource).toContain('messengerListDensityOptions')
    expect(messengerListSource).toContain('MessengerListItem')
    expect(messengerListSource).toContain('TreeSelect')
    expect(messengerListSource).toContain('DropdownMenu')
    expect(messengerListSource).toContain('searchValue')
    expect(messengerListSource).toContain('messenger-list-search')
    expect(messengerListSource).toContain('visibleItems')
    expect(messengerListSource).toContain("t('List display')")
    expect(messengerListSource).not.toContain("t('New message')")
    expect(messengerListItemSource).toContain('MessengerListItemModel')
    expect(messengerListItemSource).toContain(
      'density?: MessengerListItemDensity'
    )
    expect(messengerActionSource).toContain('Messenger conversation actions')
    expect(messengerActionSource).toContain('ButtonGroup')
    expect(messengerActionSource).toContain('Spacer')
    expect(messengerActionSource).toContain('toggle-inspector')
    expect(messengerViewerSource).toContain('MessageBubble')
    expect(messengerViewerSource).toContain('MessengerRichEditor')
    expect(messengerViewerSource).not.toContain('ChatInput')
    expect(messengerRichEditorSource).toContain('TelegramMessengerRichEditor')
    expect(messengerRichEditorSource).toContain('WhatsAppMessengerRichEditor')
    expect(messengerRichEditorSource).toContain('SignalMessengerRichEditor')
    expect(messengerRichEditorSource).toContain('conversation.channelKind')
    expect(messengerProviderRichEditorSource).toContain('RichTextEditor')
    expect(messengerProviderRichEditorSource).toContain('select-capability')
    expect(messengerProviderRichEditorSource).toContain('preset.primaryActions')
    expect(telegramMessengerRichEditorSource).toContain(
      'telegramMessengerComposerPreset'
    )
    expect(whatsAppMessengerRichEditorSource).toContain(
      'whatsAppMessengerComposerPreset'
    )
    expect(signalMessengerRichEditorSource).toContain(
      'signalMessengerComposerPreset'
    )
    expect(messengerComposerSource).toContain('telegramMessengerComposerPreset')
    expect(messengerComposerSource).toContain('whatsAppMessengerComposerPreset')
    expect(messengerComposerSource).toContain('signalMessengerComposerPreset')
    expect(messengerComposerSource).toContain('messengerComposerVariantLabel')
    expect(messengerComposerSource).toContain("conversation.kind === 'group'")
    expect(messengerMessageSource).toContain('MessengerAction')
    expect(messengerMessageSource).toContain('MessengerViewer')
    expect(messengerInspectorSource).toContain('MessengerInspectorModel')
    expect(messengerInspectorSource).toContain("t('Messenger Intelligence')")
    expect(messengerInspectorSource).toContain('ScoreGauge')
    expect(messengerWorkspaceSource).toContain('MessengerList')
    expect(messengerWorkspaceSource).toContain('MessengerMessage')
    expect(messengerWorkspaceSource).toContain('MessengerInspector')
    expect(mailMessageSource).toContain(':attachments="message.attachments"')
    expect(mailMessageSource).not.toContain(
      'communication-email-center__context'
    )
    expect(mailViewerSource).not.toContain('attributeGroups')
    expect(mailViewerSource).not.toContain('hermesEntities')
    expect(mailViewerSource).not.toContain("t('Message attributes')")
    expect(mailViewerSource).not.toContain("t('Hermes candidates')")
    expect(mailQuotedOriginalSource).toContain('Original message')
    expect(mailReplyComposerSource).toContain('Reply draft')
    expect(mailWorkspaceSource).toContain('MailList')
    expect(mailWorkspaceSource).toContain(':search-query="searchQuery"')
    expect(mailWorkspaceSource).toContain("@update-search-query=\"emit('update-search-query', $event)\"")
    expect(mailWorkspaceSource).toContain('MailMessage')
    expect(mailWorkspaceSource).not.toContain('MailThread')
    expect(mailWorkspaceSource).toContain('activeMessage')
    expect(mailWorkspaceSource).toContain(
      'MailInspector v-if="isInspectorVisible"'
    )
    expect(mailWorkspaceSource).not.toContain('CommunicationHermesInspector')
    expect(mailWorkspaceSource).toContain(
      '@toggle-inspector="handleToggleInspector"'
    )
    expect(conversationSource).toContain('ChatInput')
    expect(conversationSource).toContain('MailMessage')
    expect(conversationSource).not.toContain('MailThread')
    expect(inspectorSource).toContain('Entities stay candidates')
    expect(channelWorkspaceSource).toContain('ChannelWorkspace')
    expect(channelListSource).toContain('Channel rooms')
    expect(channelListSource).toContain('TreeSelect')
    expect(channelListSource).toContain('Channel provider')
    expect(channelListSource).toContain('Direct chats')
    expect(channelListSource).toContain('communication-channel-direct-folder')
    expect(channelListSource).toContain('select-direct-chat')
    expect(channelListSource).toContain('communicationChannelDirectChatCount')
    expect(domainCssSource).toContain(
      '.communication-channel-workspace {\n\tgrid-template-columns'
    )
    expect(domainCssSource).toContain('\theight: 100%;')
    expect(domainCssSource).toContain(
      '.communication-channel-rail__sections {\n\tdisplay: flex;'
    )
    expect(domainCssSource).toContain('.communication-calls-list__permanent')
    expect(domainCssSource).toContain('.communication-calls-list__date-section')
    expect(domainCssSource).toContain(
      '.communication-channel-rail__section[open]'
    )
    expect(domainCssSource).toContain('overscroll-behavior: contain;')
    expect(channelActionSource).toContain('actionGroups')
    expect(channelActionSource).toContain('group.actions')
    expect(channelActionSource).toContain(
      'action.contract ?? action.description'
    )
    expect(channelViewerSource).toContain('channelMessageAuthor')
    expect(channelViewerSource).toContain(':direction="message.direction"')
    expect(domainCssSource).toContain(
      '.communication-channel-composer__tools::-webkit-scrollbar'
    )
    expect(domainCssSource).toContain(
      '.communication-channel-composer__tool.hermes-icon-button'
    )
    expect(domainCssSource).toContain('height: 30px;')
    expect(domainCssSource).toContain(
      '.communication-channel-stream {\n\tgrid-template-rows: auto auto minmax(0, 1fr) auto;'
    )
    expect(domainCssSource).toContain(
      '.communication-channel-stream__messages {\n\tdisplay: grid;'
    )
    expect(domainCssSource).toContain('\tmin-height: 0;')
    expect(domainCssSource).not.toContain(
      'grid-template-rows: auto auto auto minmax(0, 1fr) auto;'
    )
    expect(channelInspectorSource).toContain(
      'CommunicationChannelInspectorModel'
    )
    expect(channelInspectorSource).toContain('ScoreGauge')
    expect(channelInspectorSource).toContain('Channel Intelligence')
    expect(channelInspectorSource).toContain('Extracted entities')
    expect(channelInspectorSource).toContain('Suggested actions')
    expect(channelInspectorSource).toContain('Related context')
    expect(channelInspectorSource).not.toContain(
      'CommunicationHermesInspectorSectionModel'
    )
    expect(channelWorkspaceComponentSource).toContain('ChannelMessage')
    expect(channelWorkspaceComponentSource).toContain('ChannelInspector')
    expect(channelWorkspaceComponentSource).toContain(
      ':model="workspace.inspector"'
    )
    expect(channelWorkspaceComponentSource).toContain('direct-chat-folders')
    expect(channelSurfaceAdaptersSource).toContain(
      'channelProviderOptionsFromSubSurfaces'
    )
    expect(channelSurfaceAdaptersSource).toContain(
      'channelActionGroupsFromSubSurface'
    )
    expect(channelSurfaceAdaptersSource).toContain(
      'channelComposerCapabilitiesFromSubSurface'
    )
    expect(communicationSurfaceSource).toContain('CommunicationSurface')
    expect(communicationSurfaceSource).toContain('CommunicationSubSurface')
    expect(communicationSurfaceSource).toContain('commonCapabilities')
    expect(communicationSurfaceSource).toContain('subSurfaces')
    expect(callViewerSource).toContain('Call transcript')
    expect(shellSource).not.toContain('frontend/src/integrations')
    expect(outboxSource).toContain('communicationOutboxCardPresentation')

    expect(primitiveStorySource).toContain('Hermes UI/General/Communication')
    expect(primitiveStorySource).not.toContain('Hermes App/Communications')
    expect(mailStorySource).toContain('Hermes App/Communications/Mail')
    expect(mailStorySource).toContain('MailListItemModel')
    expect(mailStorySource).toContain('MailListItem')
    expect(mailStorySource).toContain('MailListItemDynamics')
    expect(mailStorySource).toContain('MailListItemDensityModes')
    expect(mailStorySource).toContain('MailListItemInvariants')
    expect(mailStorySource).toContain('MailFolders')
    expect(mailStorySource).toContain('mailOpenMessage')
    expect(mailStorySource).toContain('Message identity')
    expect(mailStorySource).toContain('Message actions')
    expect(mailStorySource).toContain('Hermes actions')
    expect(mailStorySource).toContain('deleteMessageFromProvider')
    expect(mailStorySource).toContain('runWorkflowAction:create_task')
    expect(mailStorySource).toContain('MailFolderListComponent')
    expect(mailStorySource).toContain('mailStandardFolders')
    expect(mailStorySource).toContain('mailFolderChildrenById')
    expect(mailStorySource).toContain('inbox-work')
    expect(mailStorySource).toContain('archive-vendors')
    expect(mailStorySource).toContain('mailListItemMarkerOptions')
    expect(mailStorySource).toContain('mailListItemDensityOptions')
    expect(mailStorySource).toContain('timelineHint')
    expect(mailStorySource).toContain('insights')
    expect(mailStorySource).toContain('Spam and risk')
    expect(mailStorySource).toContain('MailList')
    expect(mailStorySource).toContain('MailActionComponent')
    expect(mailStorySource).toContain('MailViewerComponent')
    expect(mailStorySource).toContain('MailFooterComponent')
    expect(mailStorySource).toContain('MailInspectorComponent')
    expect(mailStorySource).toContain('mailInspectorModel')
    expect(mailStorySource).toContain('export const MailInspector')
    expect(mailStorySource).toContain('export const MailAction')
    expect(mailStorySource).toContain('export const MailViewer')
    expect(mailStorySource).toContain('export const MailFooter')
    expect(mailStorySource).toContain('MailMessage')
    expect(mailStorySource).toContain('MailReplyComposer')
    expect(mailStorySource).not.toContain('MailThread')
    expect(mailStorySource).toContain('MailWorkspace')
    expect(mailStorySource).toContain('Original message')
    expect(messengerStorySource).toContain(
      'Hermes App/Communications/Messengers'
    )
    expect(messengerStorySource).toContain('MessengerListItem')
    expect(messengerStorySource).toContain('MessengerList')
    expect(messengerStorySource).toContain('MessengerAction')
    expect(messengerStorySource).toContain('MessengerViewer')
    expect(messengerStorySource).toContain('TelegramRichEditor')
    expect(messengerStorySource).toContain('WhatsAppRichEditor')
    expect(messengerStorySource).toContain('SignalRichEditor')
    expect(messengerStorySource).toContain('MessengerMessage')
    expect(messengerStorySource).toContain('MessengerInspector')
    expect(messengerStorySource).toContain('MessengerWorkspace')
    expect(messengerStorySource).not.toContain('CommunicationWorkspaceShell')
    expect(channelStorySource).toContain('Hermes App/Communications/Channels')
    expect(channelStorySource).toContain('CommunicationChannelWorkspace')
    expect(channelStorySource).toContain('ChannelListItem')
    expect(channelStorySource).toContain('ChannelList')
    expect(channelStorySource).toContain('ChannelAction')
    expect(channelStorySource).not.toContain('ChannelBackendSurface')
    expect(channelStorySource).not.toContain('ProviderSetupSurface')
    expect(channelStorySource).not.toContain('EventIngestSurface')
    expect(channelStorySource).not.toContain('BackendSurfaces')
    expect(channelStorySource).toContain('ChannelViewer')
    expect(channelStorySource).toContain('ChannelInspector')
    expect(channelStorySource).toContain('ChannelMessage')
    expect(channelStorySource).toContain('ChannelWorkspace')
    expect(channelStorySource).toContain('storybook-canvas--workspace')
    expect(channelStorySource).not.toContain('height: 760px')
    expect(storybookCssSource).toContain(
      '.storybook-canvas--workspace > .communication-channel-workspace'
    )
    expect(channelStorySource).toContain('directChatFolders')
    expect(channelStorySource).toContain('Owner conversations')
    expect(channelStorySource).not.toContain(
      'Provider-neutral channel surface for Zulip now'
    )
    expect(channelStorySource).not.toContain('Zulip SubSurface')
    expect(channelStorySource).not.toContain('capabilityInspectorItems')
    expect(channelStorySource).toContain('Channel confidence')
    expect(channelStorySource).toContain('Source evidence linked')
    expect(channelStorySource).toContain('Export SLA confirmation')
    expect(channelStorySource).toContain('Ask in direct chat')
    expect(channelStorySource).toContain('Zulip')
    expect(channelStorySource).toContain('channels:zulip')
    expect(channelStorySource).toContain('channelFixtureSurfaces')
    expect(channelStorySource).not.toContain('useCommunicationsWorkspaceSurface')
    expect(channelStorySource).toContain('channelActionGroupsFromSubSurface')
    expect(channelStorySource).toContain(
      'channelComposerCapabilitiesFromSubSurface'
    )
    expect(channelStorySource).toContain(
      'channelProviderOptionsFromSubSurfaces'
    )
    expect(callsStorySource).toContain('Hermes App/Communications/Calls')
    expect(callsStorySource).toContain('CommunicationCallsSurface')
    expect(callsStorySource).toContain('CallListItem')
    expect(callsStorySource).toContain('CallList')
    expect(callsStorySource).toContain('CallAction')
    expect(callsStorySource).toContain('CallViewer')
    expect(callsStorySource).toContain('CallInspector')
    expect(callsStorySource).toContain('CallMessage')
    expect(callsStorySource).toContain('CallWorkspace')
    expect(callsStorySource).toContain('storybook-canvas--workspace')
    expect(callsStorySource).toContain('Zoom')
    expect(callsStorySource).toContain('Yandex Telemost')
    expect(callsStorySource).toContain('Zulip')
    expect(callsStorySource).toContain('permanentMeetings')
    expect(callsStorySource).toContain('Product review room')
    expect(callsStorySource).toContain('Ops incident room')
    expect(callsStorySource).toContain('Today')
    expect(callsStorySource).toContain('Yesterday')
    expect(callsStorySource).not.toContain("providerKind: 'mail'")
    expect(callsStorySource).not.toContain("channelKind: 'mail'")
    expect(callsSurfaceSource).toContain('CallWorkspace')
    expect(callListItemSource).toContain('communicationCallProviderIconName')
    expect(callListSource).toContain('Permanent meetings')
    expect(callListSource).toContain('Calls by date')
    expect(callListSource).toContain('dateGroups')
    expect(callListSource).toContain('createCommunicationCallDateGroups')
    expect(callListSource).not.toContain('sortKey')
    expect(callElementsSource).toContain('sortKey')
    expect(callElementsSource).toContain('createCommunicationCallDateGroups')
    expect(callListSource).not.toContain('Recurring rooms')
    expect(callListSource).not.toContain('Recordings')
    expect(callActionSource).toContain('Call actions')
    expect(callActionSource).toContain('Create meeting')
    expect(callViewerSource).toContain('Call transcript')
    expect(callViewerSource).toContain('Call recordings')
    expect(callInspectorSource).toContain('Call Intelligence')
    expect(callInspectorSource).toContain('ScoreGauge')
    expect(callMessageSource).toContain('CallAction')
    expect(callMessageSource).toContain('CallViewer')
    expect(callWorkspaceSource).toContain('communication-calls-workspace')
    expect(storybookCssSource).toContain(
      '.storybook-canvas--workspace > .communication-calls-workspace'
    )
    expect(storySources.join('\n')).not.toContain(
      'Hermes UI/Domain/Communications'
    )
    expect(storySources.join('\n')).not.toContain('/api/v1/')
    expect(storySources.join('\n')).not.toContain('/api/v1/telegram')
    expect(storySources.join('\n')).not.toContain('/api/v1/whatsapp')
  })
})
