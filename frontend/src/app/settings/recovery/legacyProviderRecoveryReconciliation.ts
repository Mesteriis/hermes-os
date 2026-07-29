import type {
	LegacyProviderRecoveryCandidateV1,
} from '../../../platform/legacy-recovery'
import type {
	LegacyProviderCandidateProgressV1,
} from './legacyProviderRecoveryPresentation'

export function legacyProviderRecoveryInitialProgressV1(
	candidates: readonly LegacyProviderRecoveryCandidateV1[],
): Record<string, LegacyProviderCandidateProgressV1> {
	return Object.fromEntries(
		candidates.map((candidate) => [candidate.sourceHandle, 'pending']),
	)
}

export function legacyProviderRecoveryQueueV1(
	candidates: readonly LegacyProviderRecoveryCandidateV1[],
): LegacyProviderRecoveryCandidateV1[] {
	const order = { icloud: 0, gmail: 1, telegram_user: 2 } as const
	return [...candidates].sort(
		(left, right) => order[left.kind] - order[right.kind],
	)
}
