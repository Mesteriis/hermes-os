import type { CommunicationAccountOption } from '../../types/communications'

export type ComposeEdgePanelId = 'ai' | 'context'

export type ComposeEdgePanelAction = {
  id: string
  label: string
  icon: string
  description: string
  disabled?: boolean
}

export type ComposeEdgePanelSection = {
  id: string
  title: string
  icon: string
  items: string[]
}

export function sendCapableComposeAccounts(
  accounts: readonly CommunicationAccountOption[]
): CommunicationAccountOption[] {
  return accounts.filter((account) => account.can_send)
}

export function composeAccountOptionSignature(
  accounts: readonly CommunicationAccountOption[]
): string {
  return accounts
    .map((account) => `${account.account_id}:${account.can_send ? 'send' : 'read'}`)
    .join('\u0000')
}

export function composeAiPanelActions(): ComposeEdgePanelAction[] {
  return [
    {
      id: 'prompt',
      label: 'Prompt to email',
      icon: 'tabler:sparkles',
      description: 'Draft from intent'
    },
    {
      id: 'rewrite',
      label: 'Rewrite draft',
      icon: 'tabler:refresh-dot',
      description: 'Keep meaning, improve shape'
    },
    {
      id: 'tone',
      label: 'Adjust tone',
      icon: 'tabler:mood-smile',
      description: 'Make it warmer, firmer, or shorter'
    },
    {
      id: 'translate',
      label: 'Translate',
      icon: 'tabler:language',
      description: 'Prepare another language version'
    },
    {
      id: 'correct',
      label: 'Autocorrect',
      icon: 'tabler:writing-sign',
      description: 'Fix typos and grammar'
    }
  ]
}

export function composeContextPanelSections(
  accounts: readonly CommunicationAccountOption[]
): ComposeEdgePanelSection[] {
  const senders = accounts
    .filter((account) => account.can_send)
    .map((account) => account.email || account.label)
  return [
    {
      id: 'templates',
      title: 'Templates',
      icon: 'tabler:template',
      items: ['Quick reply', 'Follow-up', 'Intro']
    },
    {
      id: 'signatures',
      title: 'Signatures',
      icon: 'tabler:signature',
      items: ['Default signature', 'Short signature']
    },
    {
      id: 'recipients',
      title: 'Recipient review',
      icon: 'tabler:users',
      items: senders.length > 0 ? senders : ['Select sender']
    },
    {
      id: 'safety',
      title: 'Safety checks',
      icon: 'tabler:shield-check',
      items: ['Sender available', 'Address fields checked']
    }
  ]
}
