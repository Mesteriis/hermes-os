import type { LegacyProviderRecoveryCandidateV1 } from '../../../platform/legacy-recovery'

export type LegacyProviderCandidateProgressV1 =
	| 'pending'
	| 'running'
	| 'completed'
	| 'failed'

export type LegacyProviderRecoveryRowV1 = {
	key: string
	label: string
	position: number
	state: LegacyProviderCandidateProgressV1
}

export function legacyProviderRecoveryFingerprintV1(value?: string): string {
	return value ? `${value.slice(0, 12)}…${value.slice(-12)}` : 'Not inspected'
}

export function legacyProviderRecoveryRowsV1(
	candidates: readonly LegacyProviderRecoveryCandidateV1[] | undefined,
	progress: Readonly<Record<string, LegacyProviderCandidateProgressV1>>,
): LegacyProviderRecoveryRowV1[] {
	return candidates?.map((candidate, index) => ({
		key: candidate.sourceHandle,
		label: candidateLabel(candidate.kind),
		position: index + 1,
		state: progress[candidate.sourceHandle] ?? 'pending',
	})) ?? []
}

export function legacyProviderRecoveryCompletedV1(
	progress: Readonly<Record<string, LegacyProviderCandidateProgressV1>>,
): boolean {
	const states = Object.values(progress)
	return states.length > 0 && states.every((state) => state === 'completed')
}

function candidateLabel(
	kind: LegacyProviderRecoveryCandidateV1['kind'],
): string {
	switch (kind) {
		case 'gmail':
			return 'Gmail account'
		case 'icloud':
			return 'iCloud Mail account'
		case 'telegram_user':
			return 'Telegram user account'
	}
}
