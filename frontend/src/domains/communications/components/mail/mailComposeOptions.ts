import type { CommunicationAccountOption } from '../../types/communications'

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
