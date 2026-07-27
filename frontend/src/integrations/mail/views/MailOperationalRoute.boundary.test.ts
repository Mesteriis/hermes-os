import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('Mail operational active route boundary', () => {
	it('uses separate Mail read, composition, sync, delivery and status contracts without domain coupling', () => {
		const route = read('../views/MailOperationalRoute.vue')
		const compositionController = read('../queries/useMailComposition.ts')
		const draftController = read('../queries/useMailDrafts.ts')
		const templateController = read('../queries/useMailTemplates.ts')
		const signatureController = read('../queries/useMailSignatures.ts')
		const deliveryController = read('../queries/useMailDelivery.ts')
		const syncController = read('../queries/useMailSync.ts')
		const readController = read('../queries/useMailOperationalRead.ts')
		const flagController = read('../queries/useMailMessageFlags.ts')
		const flagGateway = read('../api/mailMessageFlagsGateway.ts')
		const flagCommandClient = read('../api/mailMessageFlagCommandClient.ts')
		const flagQueryClient = read('../api/mailMessageFlagQueryClient.ts')
		const gateway = read('../api/mailOperationalGateway.ts')
		const compositionGateway = read('../api/mailCompositionGateway.ts')
		const compositionCommandClient = read('../api/mailCompositionCommandClient.ts')
		const compositionQueryClient = read('../api/mailCompositionQueryClient.ts')
		const compositionConnections = read('../queries/mailCompositionConnections.ts')
		const readGateway = read('../api/mailOperationalReadGateway.ts')
		const readClient = read('../api/mailOperationalQueryClient.ts')
		const healthClient = read('../api/mailSyncHealthClient.ts')
		const healthGateway = read('../api/mailSyncHealthGateway.ts')
		const healthConnections = read('../queries/mailSyncHealthConnections.ts')
		const healthController = read('../queries/useMailSyncHealth.ts')
		const healthModel = read('../presentation/mailSyncHealthModel.ts')
		const healthPresentation = read('../presentation/MailSyncHealthPanel.vue')
		const presentation = read('../presentation/MailOperationalPage.vue')
		const compositionPresentation = read('../presentation/MailCompositionPanel.vue')
		const draftPresentation = read('../presentation/MailDraftComposer.vue')
		const templatePresentation = read('../presentation/MailTemplateLibrary.vue')
		const signaturePresentation = read('../presentation/MailSignatureLibrary.vue')
		const deliveryPresentation = read('../presentation/MailDeliveryPanel.vue')
		const readPresentation = read('../presentation/MailOperationalReadPanel.vue')
		const flagPresentation = read('../presentation/MailMessageFlagActions.vue')
		const appLayout = read('../../../app/layout/AppLayoutRoot.vue')
		const compiledAdapters = read('../../../app/client-surfaces/compiledClientSurfaceAdapters.ts')

		for (const source of [
			route,
			compositionController,
			draftController,
			templateController,
			signatureController,
			deliveryController,
			syncController,
			readController,
			flagController,
			flagGateway,
			flagCommandClient,
			flagQueryClient,
			gateway,
			compositionGateway,
			compositionCommandClient,
			compositionQueryClient,
			compositionConnections,
			readGateway,
			readClient,
			healthClient,
			healthGateway,
			healthConnections,
			healthController,
			healthModel,
			healthPresentation,
			presentation,
			compositionPresentation,
			draftPresentation,
			templatePresentation,
			signaturePresentation,
			deliveryPresentation,
			readPresentation,
			flagPresentation,
		]) {
			expect(source).not.toMatch(/\/api\/v1\//)
			expect(source).not.toMatch(/domains\/communications/)
			expect(source).not.toMatch(/integrations\/(telegram|whatsapp|zulip)/)
		}
		expect(gateway).toContain('getMailSyncConnectClient')
		expect(gateway).toContain('getMailDeliveryCommandConnectClient')
		expect(gateway).toContain('getMailDeliveryQueryConnectClient')
		expect(compositionGateway).toContain('MailCompositionCommandV1Schema')
		expect(compositionGateway).toContain('MailCompositionQueryV1Schema')
		expect(compositionController).toContain('useMailDrafts')
		expect(compositionController).toContain('useMailTemplates')
		expect(compositionController).toContain('useMailSignatures')
		expect(draftController).toContain('upsertMailDraft')
		expect(templateController).toContain('previewMailTemplate')
		expect(signatureController).toContain('upsertMailSignature')
		expect(compositionCommandClient).toContain('MailCompositionCommandService')
		expect(compositionQueryClient).toContain('MailCompositionQueryService')
		expect(compositionConnections).toContain("'mail.composition.query.v1'")
		expect(readClient).toContain('MailOperationalQueryService')
		expect(readGateway).toContain('MailOperationalQueryV1Schema')
		expect(flagGateway).toContain('MailMessageFlagCommandV1Schema')
		expect(flagGateway).toContain('MailMessageFlagStatusRequestV1Schema')
		expect(flagCommandClient).toContain('MailMessageFlagCommandService')
		expect(flagQueryClient).toContain('MailMessageFlagQueryService')
		expect(flagController).toContain('mutateMailMessageFlag')
		expect(flagController).toContain('getMailMessageFlagStatus')
		expect(healthClient).toContain('MailSyncHealthQueryService')
		expect(healthGateway).toContain('MailSyncHealthQueryV1Schema')
		expect(healthConnections).toContain("'mail.sync.health.query.v1'")
		expect(presentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(compositionPresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(draftPresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(templatePresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(signaturePresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(deliveryPresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(readPresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(flagPresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(healthPresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(appLayout).toContain('MailOperationalRoute')
		expect(appLayout).toContain("'mail.delivery.v1'")
		expect(appLayout).toContain("'mail.composition.command.v1'")
		expect(appLayout).toContain("'mail.composition.query.v1'")
		expect(appLayout).toContain("'mail.operational.query.v1'")
		expect(appLayout).toContain("'mail.message-flags.command.v1'")
		expect(appLayout).toContain("'mail.message-flags.query.v1'")
		expect(appLayout).toContain("'mail.sync.v1'")
		expect(appLayout).toContain("'mail.sync.health.query.v1'")
		expect(compiledAdapters).toContain("'mail-integration'")
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
