import { describe, expect, it } from 'vitest'

import {
	authorizationView,
	buildTelegramAccountRows,
} from './telegramAccountAccessModel'

describe('Telegram account access presentation model', () => {
	it('maps lifecycle projections without transport objects', () => {
		expect(buildTelegramAccountRows([{
			accountId: 'account-1',
			displayName: 'Personal',
			state: 'active',
			runtimeState: 'ready',
		} as never], 'account-1')).toEqual([{
			id: 'account-1',
			title: 'Personal',
			detail: 'active · ready',
			selected: true,
		}])
	})

	it('keeps authorization secrets out of the model mapper', () => {
		expect(authorizationView({
			state: 'waiting_password',
			passwordHint: 'two words',
		})).toEqual({
			state: 'waiting_password',
			passwordHint: 'two words',
			qrLink: '',
		})
		expect(authorizationView(null).state).toBe('unknown')
	})
})
