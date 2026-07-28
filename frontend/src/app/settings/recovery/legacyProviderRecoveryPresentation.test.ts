import { describe, expect, it } from 'vitest'

import type { LegacyProviderRecoveryCandidateV1 } from '../../../platform/legacy-recovery'
import {
	legacyProviderRecoveryCompletedV1,
	legacyProviderRecoveryFingerprintV1,
	legacyProviderRecoveryRowsV1,
} from './legacyProviderRecoveryPresentation'

const candidates: readonly LegacyProviderRecoveryCandidateV1[] = [
	{ sourceHandle: 'a'.repeat(64), kind: 'icloud', state: 'ready_to_apply' },
	{ sourceHandle: 'b'.repeat(64), kind: 'gmail', state: 'reauthorization_required' },
	{ sourceHandle: 'c'.repeat(64), kind: 'telegram_user', state: 'qr_authorization_required' },
]

describe('legacy provider recovery presentation', () => {
	it('projects only sanitized candidate rows', () => {
		expect(legacyProviderRecoveryRowsV1(candidates, {
			[candidates[0]!.sourceHandle]: 'completed',
		})).toEqual([
			{
				key: candidates[0]!.sourceHandle,
				label: 'iCloud Mail account',
				position: 1,
				state: 'completed',
			},
			{
				key: candidates[1]!.sourceHandle,
				label: 'Gmail account',
				position: 2,
				state: 'pending',
			},
			{
				key: candidates[2]!.sourceHandle,
				label: 'Telegram user account',
				position: 3,
				state: 'pending',
			},
		])
	})

	it('requires a non-empty all-completed progress set', () => {
		expect(legacyProviderRecoveryCompletedV1({})).toBe(false)
		expect(legacyProviderRecoveryCompletedV1({
			one: 'completed',
			two: 'completed',
		})).toBe(true)
		expect(legacyProviderRecoveryCompletedV1({
			one: 'completed',
			two: 'failed',
		})).toBe(false)
	})

	it('formats only a bounded fingerprint preview', () => {
		expect(legacyProviderRecoveryFingerprintV1()).toBe('Not inspected')
		expect(legacyProviderRecoveryFingerprintV1('a'.repeat(64)))
			.toBe(`${'a'.repeat(12)}…${'a'.repeat(12)}`)
	})
})
