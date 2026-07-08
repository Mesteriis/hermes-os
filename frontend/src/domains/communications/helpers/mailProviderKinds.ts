const mailProviderKinds = new Set(['gmail', 'icloud', 'imap'])

export function isMailProviderKind(providerKind: string | null | undefined): boolean {
  return mailProviderKinds.has(providerKind?.trim().toLowerCase() ?? '')
}
