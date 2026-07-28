import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('legacy recovery UI composes separate Mail and Telegram provider workflows', async () => {
  const [
    app,
    panel,
    sourceContract,
    sourceAdapter,
    vaultClient,
    settings,
    mail,
    telegram,
  ] = await Promise.all([
    readFile(
      new URL(
        'frontend/src/app/settings/recovery/useLegacyProviderRecovery.ts',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'frontend/src/app/settings/recovery/LegacyProviderRecoveryPanel.vue',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'frontend/src/platform/legacy-recovery/legacyProviderRecoveryHost.ts',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'frontend/src/platform/legacy-recovery/developmentLegacyProviderRecoveryHost.ts',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'frontend/src/platform/vault/ownerVaultProvisioningClient.ts',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'frontend/src/platform/settings/managedIntegrationSetup.ts',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'frontend/src/integrations/mail/recovery/mailLegacyRecoveryWorkflow.ts',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'frontend/src/integrations/telegram/recovery/telegramLegacyRecoveryWorkflow.ts',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
  ]);

  assert.match(app, /MailLegacyRecoveryWorkflowV1/);
  assert.match(app, /TelegramLegacyRecoveryWorkflowV1/);
  assert.doesNotMatch(panel, /\{\{\s*[^}]*(?:accountId|email|username|externalAccountId|apiId)/);
  assert.doesNotMatch(panel, /console\.|localStorage|sessionStorage/);

  assert.match(sourceContract, /LegacyProviderRecoverySecretPurposeV1/);
  assert.match(sourceContract, /icloud_imap_password/);
  assert.match(sourceContract, /generated_telegram_session_store_key/);
  assert.doesNotMatch(sourceContract, /legacy_telegram_session_key|oauth_token/);
  assert.match(sourceAdapter, /credentials: 'same-origin'/);
  assert.match(sourceAdapter, /cache: 'no-store'/);
  assert.match(sourceAdapter, /redirect: 'error'/);
  assert.doesNotMatch(sourceAdapter, /console\.|localStorage|sessionStorage/);

  assert.match(vaultClient, /provisionCustodied/);
  assert.match(vaultClient, /Omit<SealProvisioningHostInputV1, 'secretPayload'>/);
  assert.match(settings, /updateOperationId/);
  assert.match(settings, /applyOperationId/);

  assert.match(mail, /mailGmailPreauthorizationSettings/);
  assert.match(mail, /icloud_imap_password/);
  assert.match(mail, /reauthorization_required/);
  assert.doesNotMatch(mail, /integrations\/telegram|telegramLegacyRecovery/i);

  assert.match(telegram, /telegram_api_hash/);
  assert.match(telegram, /generated_telegram_session_store_key/);
  assert.match(telegram, /qr_authorization_required/);
  assert.doesNotMatch(telegram, /integrations\/mail|mailLegacyRecovery/i);
});
