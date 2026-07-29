import { describe, expect, it } from 'vitest'

import type {
	LegacyProviderRecoveryCandidateV1,
} from '../../../platform/legacy-recovery'
import {
	legacyProviderRecoveryInitialProgressV1,
	legacyProviderRecoveryQueueV1,
} from './legacyProviderRecoveryReconciliation'

describe('legacy provider recovery reconciliation', () => {
	it('reconciles every candidate even when the receipt has a terminal state', () => {
		const candidates: readonly LegacyProviderRecoveryCandidateV1[] = [
			{
				sourceHandle: 't'.repeat(64),
				kind: 'telegram_user',
				state: 'qr_authorization_required',
				receiptTerminalState: 'qr_authorization_required',
			},
			{
				sourceHandle: 'g'.repeat(64),
				kind: 'gmail',
				state: 'reauthorization_required',
				receiptTerminalState: 'reauthorization_required',
			},
			{
				sourceHandle: 'i'.repeat(64),
				kind: 'icloud',
				state: 'ready_to_apply',
				receiptTerminalState: 'completed',
			},
		]

		expect(legacyProviderRecoveryInitialProgressV1(candidates)).toEqual({
			[candidates[0]!.sourceHandle]: 'pending',
			[candidates[1]!.sourceHandle]: 'pending',
			[candidates[2]!.sourceHandle]: 'pending',
		})
		expect(legacyProviderRecoveryQueueV1(candidates).map(({ kind }) => kind))
			.toEqual(['icloud', 'gmail', 'telegram_user'])
	})
})
