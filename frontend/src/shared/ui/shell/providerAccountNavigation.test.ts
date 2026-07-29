import { describe, expect, it } from 'vitest'
import {
	providerAccountIdFromRoute,
	providerAccountNavigationLevel,
} from './providerAccountNavigation'

describe('provider account navigation', () => {
	it('restores the reference Telegram account level and route identifiers', () => {
		const level = providerAccountNavigationLevel('telegram', {
			channelId: 'telegram',
			entries: [{ accountId: 'personal/account', label: 'Personal Telegram' }],
			loading: false,
			selectedAccountId: 'personal/account',
		})

		expect(level.items).toEqual([
			expect.objectContaining({
				id: 'communications-telegram-accounts:all',
				label: 'Все аккаунты',
				icon: 'tabler:users',
			}),
			expect.objectContaining({
				id: 'communications-telegram-account:personal%2Faccount',
				label: 'Personal Telegram',
				icon: 'tabler:user-circle',
			}),
		])
		expect(level.currentItem.label).toBe('Personal Telegram')
		expect(providerAccountIdFromRoute(
			'telegram',
			'communications-telegram-account:personal%2Faccount',
		)).toBe('personal/account')
	})

	it('restores the reference Mail account level and a geometry-preserving loading row', () => {
		const loading = providerAccountNavigationLevel('mail')
		expect(loading.currentItem).toMatchObject({
			disabled: true,
			loading: true,
		})

		const ready = providerAccountNavigationLevel('mail', {
			channelId: 'mail',
			entries: [{ accountId: 'work@example.com', label: 'work@example.com' }],
			loading: false,
			selectedAccountId: '',
		})
		expect(ready.items).toEqual([
			expect.objectContaining({
				id: 'communications-mail-accounts:all',
				label: 'Все ящики',
				icon: 'tabler:inbox',
			}),
			expect.objectContaining({
				id: 'communications-mail-account:work%40example.com',
				icon: 'tabler:mail-opened',
			}),
		])
		expect(providerAccountIdFromRoute('mail', 'communications-mail-accounts:all')).toBe('')
	})
})
