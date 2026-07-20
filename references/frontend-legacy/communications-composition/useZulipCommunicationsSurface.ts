// Historical pre-clean-room provider surface. It is not part of the active client graph.
import { createCommunicationSubSurface } from './communicationChannelSurface'

export function useZulipCommunicationsSurface() {
  return createCommunicationSubSurface({
    channelId: 'zulip',
    labelKey: 'Zulip',
    status: 'active',
    businessQueryRoot: ['communications', 'channels'] as const,
    runtimeQueryRoot: ['integrations', 'zulip', 'runtime'] as const,
    surfacePath: 'frontend/src/domains/communications/queries/useZulipCommunicationsSurface.ts',
    capabilityNotes: [
      'Zulip stream, topic and direct-message UI is represented as the Channels sub-surface.',
      'Provider writes remain provider commands and inbound events remain raw-to-accepted Communications evidence.'
    ],
    capabilityGroups: [
      {
        id: 'zulip-outbound',
        labelKey: 'Outbound commands',
        menuLabelKey: 'Open Zulip outbound commands',
        icon: 'tabler:send',
        status: 'available',
        capabilities: [
          {
            id: 'send-stream-message',
            labelKey: 'Send stream message',
            descriptionKey: 'Compose a message into a Zulip stream and topic.',
            icon: 'tabler:message-share',
            status: 'available',
            kind: 'command',
            contract: 'send_stream_message'
          },
          {
            id: 'send-direct-message',
            labelKey: 'Send direct message',
            descriptionKey: 'Send to resolved recipient emails or Zulip user ids.',
            icon: 'tabler:message',
            status: 'available',
            kind: 'command',
            contract: 'send_direct_message'
          },
          {
            id: 'upload-file',
            labelKey: 'Upload file',
            descriptionKey: 'Prepare a provider upload for later stream or direct message composition.',
            icon: 'tabler:paperclip',
            status: 'available',
            kind: 'command',
            contract: 'upload_file'
          },
          {
            id: 'send-stream-message-with-upload',
            labelKey: 'Send stream message with upload',
            descriptionKey: 'Attach a prepared upload and reference it from stream content.',
            icon: 'tabler:file-upload',
            status: 'available',
            kind: 'command',
            contract: 'send_stream_message_with_upload'
          },
          {
            id: 'send-direct-message-with-upload',
            labelKey: 'Send direct message with upload',
            descriptionKey: 'Attach a prepared upload to a direct Zulip message.',
            icon: 'tabler:message-up',
            status: 'available',
            kind: 'command',
            contract: 'send_direct_message_with_upload'
          }
        ]
      },
      {
        id: 'zulip-message-lifecycle',
        labelKey: 'Message lifecycle',
        menuLabelKey: 'Open Zulip message lifecycle actions',
        icon: 'tabler:activity',
        status: 'available',
        capabilities: [
          {
            id: 'update-message',
            labelKey: 'Update message content or topic',
            descriptionKey: 'Supports content, topic, stream id and propagate mode in one lifecycle action.',
            icon: 'tabler:edit',
            status: 'available',
            kind: 'command',
            contract: 'update_message'
          },
          {
            id: 'delete-message',
            labelKey: 'Delete message',
            descriptionKey: 'Deletion is shown as provider action plus local tombstone/evidence state.',
            icon: 'tabler:trash',
            status: 'available',
            kind: 'command',
            contract: 'delete_message'
          },
          {
            id: 'add-reaction',
            labelKey: 'Add reaction',
            descriptionKey: 'Add a Zulip reaction while preserving provider event reconciliation.',
            icon: 'tabler:mood-plus',
            status: 'available',
            kind: 'command',
            contract: 'add_reaction'
          },
          {
            id: 'remove-reaction',
            labelKey: 'Remove reaction',
            descriptionKey: 'Remove a Zulip reaction and reconcile the resulting reaction event.',
            icon: 'tabler:mood-minus',
            status: 'available',
            kind: 'command',
            contract: 'remove_reaction'
          }
        ]
      },
      {
        id: 'zulip-event-projection',
        labelKey: 'Event ingest and projection',
        menuLabelKey: 'Open Zulip event trace actions',
        icon: 'tabler:route',
        status: 'available',
        capabilities: [
          {
            id: 'raw-message-observed',
            labelKey: 'Raw message observed',
            descriptionKey: 'Raw provider message with stream, topic, sender and content provenance.',
            icon: 'tabler:message-circle',
            status: 'available',
            kind: 'projection',
            contract: 'signal.raw.zulip.message.observed'
          },
          {
            id: 'raw-reaction-observed',
            labelKey: 'Raw reaction observed',
            descriptionKey: 'Raw reaction events preserve emoji, operation and provider message id.',
            icon: 'tabler:mood-smile',
            status: 'available',
            kind: 'projection',
            contract: 'signal.raw.zulip.reaction.observed'
          },
          {
            id: 'raw-message-update-observed',
            labelKey: 'Raw message update observed',
            descriptionKey: 'Edited content and previous topic/content are kept as source evidence.',
            icon: 'tabler:message-2-share',
            status: 'available',
            kind: 'projection',
            contract: 'signal.raw.zulip.message_update.observed'
          },
          {
            id: 'raw-message-delete-observed',
            labelKey: 'Raw message delete observed',
            descriptionKey: 'Delete events produce tombstone-style channel state.',
            icon: 'tabler:tombstone',
            status: 'available',
            kind: 'projection',
            contract: 'signal.raw.zulip.message_delete.observed'
          },
          {
            id: 'accepted-zulip-message',
            labelKey: 'Accepted Zulip message',
            descriptionKey: 'Accepted Zulip messages become provider-neutral Communications channel evidence.',
            icon: 'tabler:database-import',
            status: 'available',
            kind: 'projection',
            contract: 'signal.accepted.zulip.message'
          }
        ]
      },
      {
        id: 'zulip-composer',
        labelKey: 'Composer',
        menuLabelKey: 'Open Zulip composer tools',
        icon: 'tabler:edit',
        status: 'available',
        capabilities: [
          {
            id: 'change-topic',
            labelKey: 'Change topic',
            descriptionKey: 'Move the draft to another Zulip topic before sending.',
            icon: 'tabler:message-circle-2',
            status: 'available',
            kind: 'composer'
          },
          {
            id: 'mention-participant',
            labelKey: 'Mention participant',
            descriptionKey: 'Insert a Zulip mention into the channel draft.',
            icon: 'tabler:at',
            status: 'available',
            kind: 'composer'
          },
          {
            id: 'attach-evidence',
            labelKey: 'Attach evidence',
            descriptionKey: 'Attach source evidence through the prepared upload path.',
            icon: 'tabler:paperclip',
            status: 'available',
            kind: 'composer',
            contract: 'ZulipPreparedUpload'
          },
          {
            id: 'code-block',
            labelKey: 'Code block',
            descriptionKey: 'Insert a code block into Zulip message content.',
            icon: 'tabler:code',
            status: 'available',
            kind: 'composer'
          },
          {
            id: 'create-poll',
            labelKey: 'Create poll',
            descriptionKey: 'Prepare a poll-style prompt for channels that support it.',
            icon: 'tabler:list-check',
            status: 'partial',
            kind: 'composer'
          },
          {
            id: 'scheduled-send',
            labelKey: 'Schedule send',
            descriptionKey: 'Represent scheduled send intent before provider support is finalized.',
            icon: 'tabler:clock',
            status: 'facade',
            kind: 'composer',
            disabled: true
          }
        ]
      }
    ]
  })
}
