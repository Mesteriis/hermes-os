export type CommunicationInspectorScoreTone = 'success' | 'warning' | 'danger'

export function communicationInspectorScoreUnit(maxScore: number): string {
  return `/${maxScore}`
}

export function communicationInspectorScoreTone(
  score: number,
  maxScore: number
): CommunicationInspectorScoreTone {
  const ratio = maxScore > 0 ? score / maxScore : 0
  if (ratio >= 0.8) return 'success'
  if (ratio >= 0.6) return 'warning'
  return 'danger'
}
