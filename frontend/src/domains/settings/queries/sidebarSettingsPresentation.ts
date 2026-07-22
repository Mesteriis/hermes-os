interface SidebarRuleTranslator {
  (key: string): string
}

export function buildSidebarRuleSummaries(t: SidebarRuleTranslator) {
  return [
    { text: t('Default keeps the current sidebar order'), badge: t('Preset') },
    { text: t('Communications sources stay nested'), badge: t('Context') },
    { text: t('Hidden domains stay recoverable here'), badge: t('Safe') },
    { text: t('Settings store no message content'), badge: t('Privacy') }
  ]
}
