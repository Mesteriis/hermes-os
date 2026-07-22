import type { AccountServiceRow } from './useIntegrationsSettingsSurface'
import type { useIntegrationsSettingsSurface } from './useIntegrationsSettingsSurface'
import { runSelectedIntegrationServiceModeAction } from './integrationServiceActions'

type IntegrationsSettingsSurface = ReturnType<typeof useIntegrationsSettingsSurface>

export function useIntegrationsSettingsPanelController(options: {
  surface: IntegrationsSettingsSurface
}) {
  function eventValue(event: Event): string {
    return event.target instanceof HTMLInputElement
      || event.target instanceof HTMLSelectElement
      || event.target instanceof HTMLTextAreaElement
      ? event.target.value
      : ''
  }

  function eventChecked(event: Event): boolean {
    return event.target instanceof HTMLInputElement ? event.target.checked : false
  }

  function handleAddAccount() {
    options.surface.openConnectWizard('mail')
  }

  function handleSelectAccount(accountId: string): void {
    options.surface.selectIntegration(accountId)
  }

  function handleToggleSelectedAccount(event: Event): void {
    void options.surface.handleToggleSelectedAccount(eventChecked(event))
  }

  function handleUpdateSelectedAccountLabel(event: Event): void {
    options.surface.handleSelectedAccountLabelInput(eventValue(event))
  }

  function handleSaveSelectedAccountLabel(): void {
    void options.surface.handleSaveSelectedAccountLabel()
  }

  function handleToggleSelectedService(service: AccountServiceRow, event: Event): void {
    void options.surface.handleToggleSelectedService(service.id, eventChecked(event))
  }

  function handleRunSelectedServiceNow(service: AccountServiceRow): void {
    void options.surface.handleRunSelectedServiceNow(service.id)
  }

  function handleRunSelectedServiceModeAction(service: AccountServiceRow): void {
    void runSelectedIntegrationServiceModeAction(
      service.id,
      options.surface.handleEnableSelectedContactsBidirectional,
    )
  }

  function handleOpenCredentialRecovery(): void {
    options.surface.openCredentialRecovery()
  }

  function handleOpenSelectedServiceSetup(): void {
    options.surface.openConnectWizard()
  }

  function handleExportAccount(accountId: string): void {
    void options.surface.handleExport(accountId)
  }

  function handleLogoutAccount(accountId: string): void {
    void options.surface.handleLogout(accountId)
  }

  function handleDeleteAccount(accountId: string): void {
    void options.surface.handleDelete(accountId)
  }

  function handleCloseConnectWizard(): void {
    options.surface.closeConnectWizard()
  }

  return {
    handleAddAccount,
    handleSelectAccount,
    handleToggleSelectedAccount,
    handleUpdateSelectedAccountLabel,
    handleSaveSelectedAccountLabel,
    handleToggleSelectedService,
    handleRunSelectedServiceNow,
    handleRunSelectedServiceModeAction,
    handleOpenCredentialRecovery,
    handleOpenSelectedServiceSetup,
    handleExportAccount,
    handleLogoutAccount,
    handleDeleteAccount,
    handleCloseConnectWizard,
    accountGroups: options.surface.groups,
    isConnectWizardOpen: options.surface.isConnectWizardOpen,
    connectWizardProviderId: options.surface.connectWizardProviderId,
    connectWizardSelectedAccount: options.surface.connectWizardSelectedAccount,
    hasAccounts: options.surface.hasAccounts,
    selectedAccountSummary: options.surface.selectedAccountSummary,
    activeMailAction: options.surface.activeMailAction,
  }
}
