import { duplicates, list, violation } from './validation-diagnostics.mjs';

const IMPLEMENTATION_KEYS = [
  'currentSlice',
  'productionPackageMode',
  'productionPackages',
  'workspaceDependencyAllowlist',
  'thirdPartyDependencyAllowlist',
  'forbiddenDependencies',
  'forbiddenDependencyPrefixes',
  'cargoFeaturesEnabled',
  'cargoFeatureAllowlist',
  'targetPolicy',
  'developmentProfile',
  'ownerInventory',
  'kernelProfile',
  'exitGates',
];

const RECOVERY_PRODUCTION_PACKAGES = [
  { name: 'hermes-events-protocol', role: 'platform', owner: 'events', surface: 'contract' },
  { name: 'hermes-runtime-protocol', role: 'platform', owner: 'runtime_protocol', surface: 'contract' },
  { name: 'hermes-gateway-protocol', role: 'api', owner: 'gateway', surface: 'contract' },
  { name: 'hermes-kernel-control-store', role: 'core', owner: 'kernel', surface: 'contract' },
  { name: 'hermes-kernel-control-store-sqlite', role: 'core', owner: 'kernel', surface: 'persistence' },
  { name: 'hermes-kernel', role: 'core', owner: 'kernel', surface: 'runtime' },
];

const VAULT_FOUNDATION_PRODUCTION_PACKAGES = [
  ...RECOVERY_PRODUCTION_PACKAGES,
  { name: 'hermes-vault-protocol', role: 'platform', owner: 'vault', surface: 'contract' },
  { name: 'hermes-managed-vault-client', role: 'platform', owner: 'vault', surface: 'contract' },
  { name: 'hermes-vault-key-provider', role: 'platform', owner: 'vault', surface: 'contract' },
  { name: 'hermes-vault-key-provider-file', role: 'platform', owner: 'vault', surface: 'implementation' },
  { name: 'hermes-secure-file', role: 'platform', owner: 'secure_file', surface: 'contract' },
  { name: 'hermes-vault-store-sqlcipher', role: 'platform', owner: 'vault', surface: 'persistence' },
  { name: 'hermes-vault-runtime', role: 'platform', owner: 'vault', surface: 'runtime' },
];

const CLOCK_PRODUCTION_PACKAGES = [
  ...VAULT_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'hermes-clock-protocol', role: 'platform', owner: 'clock', surface: 'contract' },
  { name: 'hermes-clock-runtime', role: 'platform', owner: 'clock', surface: 'implementation' },
];

const TELEMETRY_FOUNDATION_PRODUCTION_PACKAGES = [
  ...CLOCK_PRODUCTION_PACKAGES,
  { name: 'hermes-telemetry-protocol', role: 'platform', owner: 'telemetry', surface: 'contract' },
  { name: 'hermes-telemetry-collector', role: 'platform', owner: 'telemetry', surface: 'runtime' },
];

const STORAGE_FOUNDATION_PRODUCTION_PACKAGES = [
  ...TELEMETRY_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'hermes-storage-protocol', role: 'platform', owner: 'storage', surface: 'contract' },
  { name: 'hermes-storage-control', role: 'platform', owner: 'storage', surface: 'implementation' },
  { name: 'hermes-storage-vault', role: 'platform', owner: 'storage', surface: 'contract' },
  { name: 'hermes-storage-runtime', role: 'platform', owner: 'storage', surface: 'runtime' },
  { name: 'hermes-storage-postgres', role: 'platform', owner: 'storage', surface: 'persistence' },
  { name: 'hermes-storage-pgbouncer', role: 'platform', owner: 'storage', surface: 'implementation' },
  { name: 'hermes-storage-migrations', role: 'platform', owner: 'storage', surface: 'implementation' },
];

const NATS_FOUNDATION_PRODUCTION_PACKAGES = [
  ...STORAGE_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'hermes-events-jetstream', role: 'platform', owner: 'events', surface: 'implementation' },
  { name: 'hermes-events-authority', role: 'platform', owner: 'events', surface: 'implementation' },
  { name: 'hermes-events-authority-runtime-control', role: 'platform', owner: 'events', surface: 'implementation' },
  { name: 'hermes-events-authority-runtime', role: 'platform', owner: 'events', surface: 'runtime' },
];

const RECOVERY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  'hermes-events-protocol': [],
  'hermes-runtime-protocol': [],
  'hermes-gateway-protocol': [
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-kernel-control-store': [],
  'hermes-kernel-control-store-sqlite': [
    { name: 'hermes-kernel-control-store', kind: 'normal' },
  ],
  'hermes-kernel': [
    { name: 'hermes-gateway-protocol', kind: 'normal' },
    { name: 'hermes-kernel-control-store', kind: 'normal' },
    { name: 'hermes-kernel-control-store-sqlite', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-secure-file', kind: 'normal' },
  ],
  'hermes-secure-file': [],
};

const VAULT_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...RECOVERY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-vault-protocol': [],
  'hermes-managed-vault-client': [
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
  ],
  'hermes-vault-key-provider': [],
  'hermes-vault-key-provider-file': [
    { name: 'hermes-vault-key-provider', kind: 'normal' },
    { name: 'hermes-secure-file', kind: 'normal' },
  ],
  'hermes-vault-store-sqlcipher': [
    { name: 'hermes-vault-key-provider', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
  ],
  'hermes-vault-runtime': [
    { name: 'hermes-vault-key-provider', kind: 'normal' },
    { name: 'hermes-vault-key-provider-file', kind: 'normal' },
    { name: 'hermes-secure-file', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
    { name: 'hermes-vault-store-sqlcipher', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
};

const CLOCK_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...VAULT_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-clock-protocol': [],
  'hermes-clock-runtime': [
    { name: 'hermes-clock-protocol', kind: 'normal' },
  ],
};

const TELEMETRY_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...CLOCK_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-telemetry-protocol': [],
  'hermes-telemetry-collector': [
    { name: 'hermes-telemetry-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
};

const STORAGE_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...TELEMETRY_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-kernel': [
    ...RECOVERY_WORKSPACE_DEPENDENCY_ALLOWLIST['hermes-kernel'],
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-storage-protocol': [],
  'hermes-storage-control': [
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
  'hermes-storage-vault': [
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
  ],
  'hermes-storage-runtime': [
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-control', kind: 'normal' },
    { name: 'hermes-storage-postgres', kind: 'normal' },
    { name: 'hermes-storage-pgbouncer', kind: 'normal' },
    { name: 'hermes-storage-migrations', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
  ],
  'hermes-storage-postgres': [
    { name: 'hermes-storage-control', kind: 'normal' },
    { name: 'hermes-storage-migrations', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-storage-pgbouncer': [
    { name: 'hermes-storage-control', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-storage-migrations': [
    { name: 'hermes-storage-control', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const NATS_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...STORAGE_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-events-jetstream': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-scheduler-protocol', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
  ],
  'hermes-events-authority': [
    { name: 'hermes-events-jetstream', kind: 'normal' },
  ],
  'hermes-events-authority-runtime-control': [
    { name: 'hermes-events-authority', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-events-authority-runtime': [
    { name: 'hermes-events-authority-runtime-control', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
};

const BLOB_FOUNDATION_PRODUCTION_PACKAGES = [
  ...NATS_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'hermes-blob-protocol', role: 'platform', owner: 'blob', surface: 'contract' },
];

const BLOB_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES = [
  ...BLOB_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'hermes-blob-client-contract', role: 'platform', owner: 'blob', surface: 'contract' },
  { name: 'hermes-blob-client', role: 'platform', owner: 'blob', surface: 'contract' },
  { name: 'hermes-blob-runtime', role: 'platform', owner: 'blob', surface: 'implementation' },
  { name: 'hermes-blob-service', role: 'platform', owner: 'blob', surface: 'runtime' },
];

const SCHEDULER_PROTOCOL_FOUNDATION_PRODUCTION_PACKAGES = [
  ...BLOB_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'hermes-scheduler-protocol', role: 'platform', owner: 'scheduler', surface: 'contract' },
];

const SCHEDULER_FOUNDATION_PRODUCTION_PACKAGES = [
  ...SCHEDULER_PROTOCOL_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'hermes-scheduler', role: 'platform', owner: 'scheduler', surface: 'implementation' },
];

const SCHEDULER_PERSISTENCE_FOUNDATION_PRODUCTION_PACKAGES = [
  ...SCHEDULER_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'hermes-scheduler-persistence', role: 'platform', owner: 'scheduler', surface: 'persistence' },
];

const GATEWAY_SESSION_FOUNDATION_PRODUCTION_PACKAGES = [
  ...SCHEDULER_PERSISTENCE_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'hermes-gateway-session-contract', role: 'api', owner: 'gateway', surface: 'contract' },
  { name: 'hermes-gateway-session', role: 'api', owner: 'gateway', surface: 'implementation' },
];

const SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_PRODUCTION_PACKAGES = [
  ...GATEWAY_SESSION_FOUNDATION_PRODUCTION_PACKAGES,
];

const SCHEDULER_JETSTREAM_FOUNDATION_PRODUCTION_PACKAGES = [
  ...SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'hermes-scheduler-jetstream', role: 'platform', owner: 'scheduler', surface: 'implementation' },
];

const SCHEDULER_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES = [
  ...SCHEDULER_JETSTREAM_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'hermes-scheduler-runtime', role: 'platform', owner: 'scheduler', surface: 'runtime' },
];

const GATEWAY_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES = [
  ...SCHEDULER_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'hermes-gateway-runtime', role: 'api', owner: 'gateway', surface: 'implementation' },
];

const MAIL_COMMUNICATIONS_FOUNDATION_PRODUCTION_PACKAGES = [
  ...GATEWAY_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'hermes-mail-api', role: 'integration', owner: 'mail', surface: 'contract' },
  { name: 'hermes-mail-core', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'hermes-mail-imap', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'hermes-mail-gmail', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'hermes-mail-smtp', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'hermes-mail-persistence', role: 'integration', owner: 'mail', surface: 'persistence' },
  { name: 'hermes-mail-runtime', role: 'integration', owner: 'mail', surface: 'runtime' },
  { name: 'hermes-mail-assembly', role: 'integration', owner: 'mail', surface: 'assembly' },
  { name: 'hermes-telegram-api', role: 'integration', owner: 'telegram', surface: 'contract' },
  { name: 'hermes-telegram-core', role: 'integration', owner: 'telegram', surface: 'implementation' },
  { name: 'hermes-telegram-tdlib', role: 'integration', owner: 'telegram', surface: 'implementation' },
  { name: 'hermes-telegram-persistence', role: 'integration', owner: 'telegram', surface: 'persistence' },
  { name: 'hermes-telegram-runtime', role: 'integration', owner: 'telegram', surface: 'runtime' },
  { name: 'hermes-telegram-assembly', role: 'integration', owner: 'telegram', surface: 'assembly' },
  { name: 'hermes-whatsapp-api', role: 'integration', owner: 'whatsapp', surface: 'contract' },
  { name: 'hermes-whatsapp-core', role: 'integration', owner: 'whatsapp', surface: 'implementation' },
  { name: 'hermes-whatsapp-persistence', role: 'integration', owner: 'whatsapp', surface: 'persistence' },
  { name: 'hermes-whatsapp-runtime', role: 'integration', owner: 'whatsapp', surface: 'runtime' },
  { name: 'hermes-whatsapp-assembly', role: 'integration', owner: 'whatsapp', surface: 'assembly' },
  { name: 'hermes-zulip-api', role: 'integration', owner: 'zulip', surface: 'contract' },
  { name: 'hermes-zulip-core', role: 'integration', owner: 'zulip', surface: 'implementation' },
  { name: 'hermes-zulip-http', role: 'integration', owner: 'zulip', surface: 'implementation' },
  { name: 'hermes-zulip-persistence', role: 'integration', owner: 'zulip', surface: 'persistence' },
  { name: 'hermes-zulip-runtime', role: 'integration', owner: 'zulip', surface: 'runtime' },
  { name: 'hermes-communications-ingress', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'hermes-communications-attachment-contract', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'hermes-communications-api', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'hermes-communications-domain', role: 'domain', owner: 'communications', surface: 'implementation' },
  { name: 'hermes-communications-persistence', role: 'domain', owner: 'communications', surface: 'persistence' },
  { name: 'hermes-communications-runtime', role: 'domain', owner: 'communications', surface: 'runtime' },
  { name: 'hermes-communications-assembly', role: 'domain', owner: 'communications', surface: 'assembly' },
];

const FIRST_OWNER_PRODUCTION_PACKAGES = [
  ...GATEWAY_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'hermes-communications-ingress', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'hermes-communications-attachment-contract', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'hermes-communications-api', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'hermes-communications-domain', role: 'domain', owner: 'communications', surface: 'implementation' },
  { name: 'hermes-communications-persistence', role: 'domain', owner: 'communications', surface: 'persistence' },
  { name: 'hermes-communications-runtime', role: 'domain', owner: 'communications', surface: 'runtime' },
  { name: 'hermes-communications-assembly', role: 'domain', owner: 'communications', surface: 'assembly' },
];

const ATTACHMENT_SECURITY_ENGINE_PRODUCTION_PACKAGES = [
  ...FIRST_OWNER_PRODUCTION_PACKAGES,
  { name: 'hermes-attachment-security-contract', role: 'engine', owner: 'attachment_security', surface: 'contract' },
  { name: 'hermes-attachment-security-core', role: 'engine', owner: 'attachment_security', surface: 'implementation' },
  { name: 'hermes-attachment-security-clamav', role: 'engine', owner: 'attachment_security', surface: 'implementation' },
  { name: 'hermes-attachment-security-persistence', role: 'engine', owner: 'attachment_security', surface: 'persistence' },
  { name: 'hermes-attachment-security-runtime', role: 'engine', owner: 'attachment_security', surface: 'runtime' },
  { name: 'hermes-attachment-security-assembly', role: 'engine', owner: 'attachment_security', surface: 'assembly' },
];

const MAIL_OUTBOUND_MIME_ATTACHMENTS_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_SECURITY_ENGINE_PRODUCTION_PACKAGES,
  { name: 'hermes-mail-api', role: 'integration', owner: 'mail', surface: 'contract' },
  { name: 'hermes-mail-core', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'hermes-mail-imap', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'hermes-mail-gmail', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'hermes-mail-smtp', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'hermes-mail-persistence', role: 'integration', owner: 'mail', surface: 'persistence' },
  { name: 'hermes-mail-runtime', role: 'integration', owner: 'mail', surface: 'runtime' },
  { name: 'hermes-mail-assembly', role: 'integration', owner: 'mail', surface: 'assembly' },
];

const COMMUNICATIONS_CONTENT_READ_PRODUCTION_PACKAGES = [
  ...MAIL_OUTBOUND_MIME_ATTACHMENTS_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communications-content-api',
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  },
];

const COMMUNICATIONS_SAVED_SEARCH_PRODUCTION_PACKAGES = [
  ...COMMUNICATIONS_CONTENT_READ_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communications-saved-query-api',
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  },
];

const COMMUNICATIONS_SENDER_INSIGHTS_PRODUCTION_PACKAGES = [
  ...COMMUNICATIONS_SAVED_SEARCH_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communications-sender-insights-api',
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  },
];

const COMMUNICATIONS_EXPORT_PRODUCTION_PACKAGES = [
  ...COMMUNICATIONS_SENDER_INSIGHTS_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communications-evidence-export-source-api',
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  },
  {
    name: 'hermes-communications-export-api',
    role: 'workflow',
    owner: 'communications_export',
    surface: 'contract',
  },
  {
    name: 'hermes-communications-export-core',
    role: 'workflow',
    owner: 'communications_export',
    surface: 'implementation',
  },
  {
    name: 'hermes-communications-export-persistence',
    role: 'workflow',
    owner: 'communications_export',
    surface: 'persistence',
  },
  {
    name: 'hermes-communications-export-runtime',
    role: 'workflow',
    owner: 'communications_export',
    surface: 'runtime',
  },
  {
    name: 'hermes-communications-export-assembly',
    role: 'workflow',
    owner: 'communications_export',
    surface: 'assembly',
  },
];

const COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATIONS_EXPORT_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-delivery-intent-api',
    role: 'workflow',
    owner: 'communication_delivery_intent',
    surface: 'contract',
  },
  {
    name: 'hermes-communication-delivery-intent-core',
    role: 'workflow',
    owner: 'communication_delivery_intent',
    surface: 'implementation',
  },
];

const COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-delivery-intent-persistence',
    role: 'workflow',
    owner: 'communication_delivery_intent',
    surface: 'persistence',
  },
];

const COMMUNICATION_DELIVERY_INTENT_RUNTIME_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-delivery-intent-runtime',
    role: 'workflow',
    owner: 'communication_delivery_intent',
    surface: 'runtime',
  },
];

const COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELIVERY_INTENT_RUNTIME_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-delivery-intent-assembly',
    role: 'workflow',
    owner: 'communication_delivery_intent',
    surface: 'assembly',
  },
];

const DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_PRODUCTION_PACKAGES,
  {
    name: 'hermes-mail-delivery-intent-contract',
    role: 'integration',
    owner: 'mail',
    surface: 'contract',
  },
  {
    name: 'hermes-telegram-delivery-intent-contract',
    role: 'integration',
    owner: 'telegram',
    surface: 'contract',
  },
  {
    name: 'hermes-whatsapp-delivery-intent-contract',
    role: 'integration',
    owner: 'whatsapp',
    surface: 'contract',
  },
  {
    name: 'hermes-zulip-delivery-intent-contract',
    role: 'integration',
    owner: 'zulip',
    surface: 'contract',
  },
  {
    name: 'hermes-communication-delivery-intent-event-adapters',
    role: 'workflow',
    owner: 'communication_delivery_intent',
    surface: 'implementation',
  },
];

const COMMUNICATION_BULK_ACTION_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-bulk-action-api',
    role: 'workflow',
    owner: 'communication_bulk_action',
    surface: 'contract',
  },
  {
    name: 'hermes-communication-bulk-action-core',
    role: 'workflow',
    owner: 'communication_bulk_action',
    surface: 'implementation',
  },
];

const COMMUNICATION_BULK_ACTION_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_BULK_ACTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-bulk-action-persistence',
    role: 'workflow',
    owner: 'communication_bulk_action',
    surface: 'persistence',
  },
];

const COMMUNICATION_BULK_ACTION_RUNTIME_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_BULK_ACTION_PERSISTENCE_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-bulk-action-runtime',
    role: 'workflow',
    owner: 'communication_bulk_action',
    surface: 'runtime',
  },
];

const COMMUNICATION_BULK_ACTION_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_BULK_ACTION_RUNTIME_CORE_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-bulk-action-assembly',
    role: 'workflow',
    owner: 'communication_bulk_action',
    surface: 'assembly',
  },
];

const COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_BULK_ACTION_ASSEMBLY_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-delayed-delivery-api',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'contract',
  },
  {
    name: 'hermes-communication-delayed-delivery-core',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'implementation',
  },
];

const COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-delayed-delivery-persistence',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'persistence',
  },
];

const COMMUNICATION_DELAYED_DELIVERY_EXECUTION_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-delayed-delivery-execution',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'implementation',
  },
];

const COMMUNICATION_DELAYED_DELIVERY_EVENT_ADAPTERS_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELAYED_DELIVERY_EXECUTION_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-delayed-delivery-event-adapters',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'implementation',
  },
];

const COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELAYED_DELIVERY_EVENT_ADAPTERS_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-delayed-delivery-runtime-adapters',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'implementation',
  },
];

const COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-delayed-delivery-store-adapters',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'persistence',
  },
];

const COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-delayed-delivery-runtime',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'runtime',
  },
];

const COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-delayed-delivery-assembly',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'assembly',
  },
];

const COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-cross-channel-forward-api',
    role: 'workflow',
    owner: 'communication_cross_channel_forward',
    surface: 'contract',
  },
  {
    name: 'hermes-communication-cross-channel-forward-core',
    role: 'workflow',
    owner: 'communication_cross_channel_forward',
    surface: 'implementation',
  },
];

const COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-cross-channel-forward-persistence',
    role: 'workflow',
    owner: 'communication_cross_channel_forward',
    surface: 'persistence',
  },
];

const COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communications-cross-channel-forward-source-api',
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  },
];

const COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-delivery-intent-ingress-api',
    role: 'workflow',
    owner: 'communication_delivery_intent',
    surface: 'contract',
  },
];

const COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_PRODUCTION_PACKAGES =
  COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_PRODUCTION_PACKAGES;

const COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-cross-channel-forward-runtime',
    role: 'workflow',
    owner: 'communication_cross_channel_forward',
    surface: 'runtime',
  },
];

const COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communication-cross-channel-forward-assembly',
    role: 'workflow',
    owner: 'communication_cross_channel_forward',
    surface: 'assembly',
  },
];

const COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communications-call-evidence-ingress',
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  },
  {
    name: 'hermes-communications-call-evidence-core',
    role: 'domain',
    owner: 'communications',
    surface: 'implementation',
  },
];

const COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communications-call-evidence-persistence',
    role: 'domain',
    owner: 'communications',
    surface: 'persistence',
  },
];

const COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_PRODUCTION_PACKAGES = [
  ...COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communications-call-evidence-api',
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  },
];

const REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_PRODUCTION_PACKAGES,
  {
    name: 'hermes-review-attention-api',
    role: 'domain',
    owner: 'review',
    surface: 'contract',
  },
  {
    name: 'hermes-review-attention-core',
    role: 'domain',
    owner: 'review',
    surface: 'implementation',
  },
];

const REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  {
    name: 'hermes-review-attention-persistence',
    role: 'domain',
    owner: 'review',
    surface: 'persistence',
  },
];

const REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_PRODUCTION_PACKAGES = [
  ...REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_PRODUCTION_PACKAGES,
  {
    name: 'hermes-review-attention-runtime',
    role: 'domain',
    owner: 'review',
    surface: 'runtime',
  },
];

const REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
  {
    name: 'hermes-review-attention-assembly',
    role: 'domain',
    owner: 'review',
    surface: 'assembly',
  },
];

const COMMUNICATIONS_AI_SOURCE_CONTRACT_PRODUCTION_PACKAGES = [
  ...REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_PRODUCTION_PACKAGES,
  {
    name: 'hermes-communications-ai-source-api',
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  },
  {
    name: 'hermes-communication-reply-suggestion-api',
    role: 'workflow',
    owner: 'communication_reply_suggestion',
    surface: 'contract',
  },
  {
    name: 'hermes-communication-reply-suggestion-core',
    role: 'workflow',
    owner: 'communication_reply_suggestion',
    surface: 'implementation',
  },
  {
    name: 'hermes-communication-reply-suggestion-persistence',
    role: 'workflow',
    owner: 'communication_reply_suggestion',
    surface: 'persistence',
  },
  {
    name: 'hermes-communication-reply-suggestion-runtime',
    role: 'workflow',
    owner: 'communication_reply_suggestion',
    surface: 'runtime',
  },
  {
    name: 'hermes-communication-reply-suggestion-assembly',
    role: 'workflow',
    owner: 'communication_reply_suggestion',
    surface: 'assembly',
  },
  {
    name: 'hermes-ai-contracts',
    role: 'engine',
    owner: 'ai',
    surface: 'contract',
  },
  {
    name: 'hermes-ai-inference-core',
    role: 'engine',
    owner: 'ai',
    surface: 'implementation',
  },
  {
    name: 'hermes-ai-inference-persistence',
    role: 'engine',
    owner: 'ai',
    surface: 'persistence',
  },
  {
    name: 'hermes-ollama-ai-api',
    role: 'integration',
    owner: 'ollama',
    surface: 'contract',
  },
  {
    name: 'hermes-ollama-ai-assembly',
    role: 'integration',
    owner: 'ollama',
    surface: 'assembly',
  },
  {
    name: 'hermes-ollama-ai-core',
    role: 'integration',
    owner: 'ollama',
    surface: 'implementation',
  },
  {
    name: 'hermes-ollama-ai-http',
    role: 'integration',
    owner: 'ollama',
    surface: 'implementation',
  },
  {
    name: 'hermes-ollama-ai-persistence',
    role: 'integration',
    owner: 'ollama',
    surface: 'persistence',
  },
  {
    name: 'hermes-ollama-ai-runtime',
    role: 'integration',
    owner: 'ollama',
    surface: 'runtime',
  },
];

const ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATIONS_AI_SOURCE_CONTRACT_PRODUCTION_PACKAGES,
  {
    name: 'hermes-attachment-archive-inspection-api',
    role: 'engine',
    owner: 'attachment_archive_inspection',
    surface: 'contract',
  },
  {
    name: 'hermes-attachment-archive-inspection-ingress',
    role: 'engine',
    owner: 'attachment_archive_inspection',
    surface: 'contract',
  },
  {
    name: 'hermes-attachment-archive-inspection-core',
    role: 'engine',
    owner: 'attachment_archive_inspection',
    surface: 'implementation',
  },
  {
    name: 'hermes-attachment-archive-inspection-zip',
    role: 'engine',
    owner: 'attachment_archive_inspection',
    surface: 'implementation',
  },
];

const ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  {
    name: 'hermes-attachment-archive-inspection-persistence',
    role: 'engine',
    owner: 'attachment_archive_inspection',
    surface: 'persistence',
  },
];

const ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_PRODUCTION_PACKAGES,
  {
    name: 'hermes-attachment-archive-inspection-runtime',
    role: 'engine',
    owner: 'attachment_archive_inspection',
    surface: 'runtime',
  },
];

const ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_PRODUCTION_PACKAGES,
  {
    name: 'hermes-attachment-archive-inspection-assembly',
    role: 'engine',
    owner: 'attachment_archive_inspection',
    surface: 'assembly',
  },
];

const COMMUNICATION_SUMMARY_BUILD_UNITS_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'hermes-communication-summary-api', role: 'workflow', owner: 'communication_summary', surface: 'contract' },
  { name: 'hermes-communication-summary-core', role: 'workflow', owner: 'communication_summary', surface: 'implementation' },
  { name: 'hermes-communication-summary-persistence', role: 'workflow', owner: 'communication_summary', surface: 'persistence' },
  { name: 'hermes-communication-summary-runtime', role: 'workflow', owner: 'communication_summary', surface: 'runtime' },
  { name: 'hermes-communication-summary-assembly', role: 'workflow', owner: 'communication_summary', surface: 'assembly' },
];

const COMMUNICATION_TRANSLATION_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_SUMMARY_BUILD_UNITS_PRODUCTION_PACKAGES,
  { name: 'hermes-communication-translation-api', role: 'workflow', owner: 'communication_translation', surface: 'contract' },
  { name: 'hermes-communication-translation-core', role: 'workflow', owner: 'communication_translation', surface: 'implementation' },
];

const COMMUNICATION_TRANSLATION_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_TRANSLATION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'hermes-communication-translation-persistence', role: 'workflow', owner: 'communication_translation', surface: 'persistence' },
];

const COMMUNICATION_TRANSLATION_RUNTIME_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_TRANSLATION_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'hermes-communication-translation-runtime', role: 'workflow', owner: 'communication_translation', surface: 'runtime' },
];

const COMMUNICATION_TRANSLATION_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_TRANSLATION_RUNTIME_PRODUCTION_PACKAGES,
  { name: 'hermes-communication-translation-assembly', role: 'workflow', owner: 'communication_translation', surface: 'assembly' },
];

const COMMUNICATION_EXPLANATION_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_TRANSLATION_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'hermes-communication-explanation-api', role: 'workflow', owner: 'communication_explanation', surface: 'contract' },
  { name: 'hermes-communication-explanation-core', role: 'workflow', owner: 'communication_explanation', surface: 'implementation' },
];

const COMMUNICATION_EXPLANATION_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_EXPLANATION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'hermes-communication-explanation-persistence', role: 'workflow', owner: 'communication_explanation', surface: 'persistence' },
];

const COMMUNICATION_EXPLANATION_RUNTIME_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_EXPLANATION_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'hermes-communication-explanation-runtime', role: 'workflow', owner: 'communication_explanation', surface: 'runtime' },
];

const COMMUNICATION_EXPLANATION_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_EXPLANATION_RUNTIME_PRODUCTION_PACKAGES,
  { name: 'hermes-communication-explanation-assembly', role: 'workflow', owner: 'communication_explanation', surface: 'assembly' },
];

const COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_EXPLANATION_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'hermes-communication-recipient-suggestion-api', role: 'workflow', owner: 'communication_recipient_suggestion', surface: 'contract' },
  { name: 'hermes-communication-recipient-suggestion-core', role: 'workflow', owner: 'communication_recipient_suggestion', surface: 'implementation' },
];

const COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'hermes-communications-recipient-source-api', role: 'domain', owner: 'communications', surface: 'contract' },
];

const COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'hermes-communication-recipient-suggestion-persistence', role: 'workflow', owner: 'communication_recipient_suggestion', surface: 'persistence' },
  { name: 'hermes-communications-recipient-source-api', role: 'domain', owner: 'communications', surface: 'contract' },
];

const COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'hermes-communication-recipient-suggestion-persistence', role: 'workflow', owner: 'communication_recipient_suggestion', surface: 'persistence' },
  { name: 'hermes-communication-recipient-suggestion-runtime', role: 'workflow', owner: 'communication_recipient_suggestion', surface: 'runtime' },
  { name: 'hermes-communications-recipient-source-api', role: 'domain', owner: 'communications', surface: 'contract' },
];

const COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_PRODUCTION_PACKAGES,
  { name: 'hermes-communication-recipient-suggestion-assembly', role: 'workflow', owner: 'communication_recipient_suggestion', surface: 'assembly' },
];

const COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'hermes-communication-task-candidate-api', role: 'workflow', owner: 'communication_task_candidate_extraction', surface: 'contract' },
  { name: 'hermes-communication-task-candidate-core', role: 'workflow', owner: 'communication_task_candidate_extraction', surface: 'implementation' },
  { name: 'hermes-communications-task-source-api', role: 'domain', owner: 'communications', surface: 'contract' },
];

const COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_PRODUCTION_PACKAGES,
  { name: 'hermes-communication-task-candidate-persistence', role: 'workflow', owner: 'communication_task_candidate_extraction', surface: 'persistence' },
];

const COMMUNICATION_TASK_CANDIDATE_RUNTIME_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'hermes-communication-task-candidate-runtime', role: 'workflow', owner: 'communication_task_candidate_extraction', surface: 'runtime' },
];

const BLOB_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...NATS_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-blob-protocol': [],
};

const BLOB_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...BLOB_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-blob-client-contract': [
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-blob-client': [
    { name: 'hermes-blob-client-contract', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-blob-runtime': [
    { name: 'hermes-blob-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
  ],
  'hermes-blob-service': [
    { name: 'hermes-blob-protocol', kind: 'normal' },
    { name: 'hermes-blob-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
  ],
};

const SCHEDULER_PROTOCOL_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...BLOB_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-kernel': [
    ...BLOB_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST['hermes-kernel'],
    { name: 'hermes-scheduler-protocol', kind: 'normal' },
  ],
  'hermes-scheduler-protocol': [
    { name: 'hermes-clock-protocol', kind: 'normal' },
  ],
};

const SCHEDULER_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_PROTOCOL_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-scheduler': [
    { name: 'hermes-clock-protocol', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-scheduler-protocol', kind: 'normal' },
  ],
};

const SCHEDULER_PERSISTENCE_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-scheduler-persistence': [
    { name: 'hermes-clock-protocol', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-scheduler', kind: 'normal' },
    { name: 'hermes-scheduler-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const GATEWAY_SESSION_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_PERSISTENCE_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-kernel': [
    ...SCHEDULER_PERSISTENCE_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST['hermes-kernel'],
    { name: 'hermes-gateway-session-contract', kind: 'normal' },
  ],
  'hermes-gateway-session-contract': [],
  'hermes-gateway-session': [
    { name: 'hermes-gateway-session-contract', kind: 'normal' },
  ],
};

const SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...GATEWAY_SESSION_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
};

const SCHEDULER_JETSTREAM_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-scheduler-jetstream': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-scheduler-protocol', kind: 'normal' },
  ],
};

const SCHEDULER_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_JETSTREAM_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-scheduler-runtime': [
    { name: 'hermes-clock-protocol', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-scheduler', kind: 'normal' },
    { name: 'hermes-scheduler-jetstream', kind: 'normal' },
    { name: 'hermes-scheduler-persistence', kind: 'normal' },
    { name: 'hermes-scheduler-protocol', kind: 'normal' },
    { name: 'hermes-secure-file', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
};

const GATEWAY_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-kernel': [
    { name: 'hermes-blob-client', kind: 'normal' },
    ...SCHEDULER_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST['hermes-kernel'],
    { name: 'hermes-gateway-runtime', kind: 'normal' },
    { name: 'hermes-gateway-session', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
  ],
  'hermes-gateway-runtime': [
    { name: 'hermes-gateway-protocol', kind: 'normal' },
    { name: 'hermes-gateway-session', kind: 'normal' },
    { name: 'hermes-gateway-session-contract', kind: 'normal' },
  ],
};

const MAIL_COMMUNICATIONS_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...GATEWAY_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-mail-api': [],
  'hermes-mail-core': [
    { name: 'hermes-mail-api', kind: 'normal' },
    { name: 'hermes-communications-ingress', kind: 'normal' },
  ],
  'hermes-mail-imap': [
    { name: 'hermes-mail-core', kind: 'normal' },
    { name: 'hermes-mail-api', kind: 'normal' },
  ],
  'hermes-mail-gmail': [
    { name: 'hermes-mail-api', kind: 'normal' },
  ],
  'hermes-mail-smtp': [
    { name: 'hermes-mail-api', kind: 'normal' },
  ],
  'hermes-mail-persistence': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-mail-api', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-mail-runtime': [
    { name: 'hermes-mail-api', kind: 'normal' },
    { name: 'hermes-mail-core', kind: 'normal' },
    { name: 'hermes-mail-imap', kind: 'normal' },
    { name: 'hermes-mail-gmail', kind: 'normal' },
    { name: 'hermes-mail-smtp', kind: 'normal' },
    { name: 'hermes-mail-persistence', kind: 'normal' },
    { name: 'hermes-attachment-security-contract', kind: 'normal' },
    { name: 'hermes-communications-attachment-contract', kind: 'normal' },
    { name: 'hermes-communications-ingress', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-managed-vault-client', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
  ],
  'hermes-mail-assembly': [
    { name: 'hermes-mail-persistence', kind: 'normal' },
    { name: 'hermes-mail-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-telegram-api': [],
  'hermes-telegram-core': [
    { name: 'hermes-telegram-api', kind: 'normal' },
    { name: 'hermes-communications-ingress', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
  ],
  'hermes-telegram-tdlib': [
    { name: 'hermes-telegram-api', kind: 'normal' },
  ],
  'hermes-telegram-persistence': [
    { name: 'hermes-communications-ingress', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-telegram-api', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-telegram-runtime': [
    { name: 'hermes-blob-client-contract', kind: 'normal' },
    { name: 'hermes-communications-ingress', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-managed-vault-client', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
    { name: 'hermes-telegram-api', kind: 'normal' },
    { name: 'hermes-telegram-core', kind: 'normal' },
    { name: 'hermes-telegram-persistence', kind: 'normal' },
    { name: 'hermes-telegram-tdlib', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
    { name: 'hermes-blob-client', kind: 'normal' },
  ],
  'hermes-telegram-assembly': [
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-telegram-persistence', kind: 'normal' },
    { name: 'hermes-telegram-runtime', kind: 'normal' },
  ],
  'hermes-whatsapp-api': [],
  'hermes-whatsapp-core': [
    { name: 'hermes-communications-ingress', kind: 'normal' },
    { name: 'hermes-whatsapp-api', kind: 'normal' },
  ],
  'hermes-whatsapp-persistence': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-whatsapp-runtime': [
    { name: 'hermes-communications-ingress', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
    { name: 'hermes-whatsapp-api', kind: 'normal' },
    { name: 'hermes-whatsapp-core', kind: 'normal' },
    { name: 'hermes-whatsapp-persistence', kind: 'normal' },
  ],
  'hermes-whatsapp-assembly': [
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-whatsapp-persistence', kind: 'normal' },
    { name: 'hermes-whatsapp-runtime', kind: 'normal' },
  ],
  'hermes-zulip-api': [],
  'hermes-zulip-core': [
    { name: 'hermes-communications-ingress', kind: 'normal' },
    { name: 'hermes-zulip-api', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
  ],
  'hermes-zulip-http': [{ name: 'hermes-zulip-api', kind: 'normal' }],
  'hermes-zulip-persistence': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-zulip-api', kind: 'normal' },
  ],
  'hermes-zulip-runtime': [
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-blob-client-contract', kind: 'normal' },
    { name: 'hermes-communications-ingress', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-managed-vault-client', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-zulip-api', kind: 'normal' },
    { name: 'hermes-zulip-core', kind: 'normal' },
    { name: 'hermes-zulip-http', kind: 'normal' },
    { name: 'hermes-zulip-persistence', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
  ],
  'hermes-communications-ingress': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-communications-attachment-contract': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-communications-api': [],
  'hermes-communications-domain': [
    { name: 'hermes-communications-api', kind: 'normal' },
  ],
  'hermes-communications-persistence': [
    { name: 'hermes-communications-api', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-communications-runtime': [
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communications-attachment-contract', kind: 'normal' },
    { name: 'hermes-communications-ingress', kind: 'normal' },
    { name: 'hermes-communications-api', kind: 'normal' },
    { name: 'hermes-communications-domain', kind: 'normal' },
    { name: 'hermes-communications-persistence', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-managed-vault-client', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
  'hermes-communications-assembly': [
    { name: 'hermes-communications-persistence', kind: 'normal' },
    { name: 'hermes-communications-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const FIRST_OWNER_WORKSPACE_DEPENDENCY_ALLOWLIST = Object.fromEntries(
  FIRST_OWNER_PRODUCTION_PACKAGES.map(({ name }) => [
    name,
    MAIL_COMMUNICATIONS_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST[name],
  ]),
);

const ATTACHMENT_SECURITY_ENGINE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...FIRST_OWNER_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-attachment-security-contract': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-attachment-security-core': [
    { name: 'hermes-attachment-security-contract', kind: 'normal' },
  ],
  'hermes-attachment-security-clamav': [
    { name: 'hermes-attachment-security-contract', kind: 'normal' },
    { name: 'hermes-attachment-security-core', kind: 'normal' },
  ],
  'hermes-attachment-security-persistence': [
    { name: 'hermes-attachment-archive-inspection-ingress', kind: 'normal' },
    { name: 'hermes-attachment-security-core', kind: 'normal' },
    { name: 'hermes-communications-attachment-contract', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-attachment-security-runtime': [
    { name: 'hermes-attachment-archive-inspection-ingress', kind: 'normal' },
    { name: 'hermes-attachment-security-clamav', kind: 'normal' },
    { name: 'hermes-attachment-security-contract', kind: 'normal' },
    { name: 'hermes-attachment-security-core', kind: 'normal' },
    { name: 'hermes-attachment-security-persistence', kind: 'normal' },
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communications-attachment-contract', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
  'hermes-attachment-security-assembly': [
    { name: 'hermes-attachment-security-persistence', kind: 'normal' },
    { name: 'hermes-attachment-security-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const MAIL_OUTBOUND_MIME_ATTACHMENTS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_SECURITY_ENGINE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  ...Object.fromEntries(
    MAIL_OUTBOUND_MIME_ATTACHMENTS_PRODUCTION_PACKAGES
      .filter(({ owner }) => owner === 'mail')
      .map(({ name }) => [
        name,
        MAIL_COMMUNICATIONS_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST[name],
      ]),
  ),
};

const COMMUNICATIONS_CONTENT_READ_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...MAIL_OUTBOUND_MIME_ATTACHMENTS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communications-content-api': [],
  'hermes-communications-runtime': [
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communications-attachment-contract', kind: 'normal' },
    { name: 'hermes-communications-content-api', kind: 'normal' },
    { name: 'hermes-communications-ingress', kind: 'normal' },
    { name: 'hermes-communications-api', kind: 'normal' },
    { name: 'hermes-communications-domain', kind: 'normal' },
    { name: 'hermes-communications-persistence', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-managed-vault-client', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATIONS_SAVED_SEARCH_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CONTENT_READ_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communications-saved-query-api': [],
  'hermes-communications-runtime': [
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communications-attachment-contract', kind: 'normal' },
    { name: 'hermes-communications-content-api', kind: 'normal' },
    { name: 'hermes-communications-ingress', kind: 'normal' },
    { name: 'hermes-communications-api', kind: 'normal' },
    { name: 'hermes-communications-domain', kind: 'normal' },
    { name: 'hermes-communications-persistence', kind: 'normal' },
    { name: 'hermes-communications-saved-query-api', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-managed-vault-client', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATIONS_SENDER_INSIGHTS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_SAVED_SEARCH_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communications-sender-insights-api': [],
  'hermes-communications-runtime': [
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communications-attachment-contract', kind: 'normal' },
    { name: 'hermes-communications-content-api', kind: 'normal' },
    { name: 'hermes-communications-ingress', kind: 'normal' },
    { name: 'hermes-communications-api', kind: 'normal' },
    { name: 'hermes-communications-domain', kind: 'normal' },
    { name: 'hermes-communications-persistence', kind: 'normal' },
    { name: 'hermes-communications-saved-query-api', kind: 'normal' },
    { name: 'hermes-communications-sender-insights-api', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-managed-vault-client', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATIONS_EXPORT_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_SENDER_INSIGHTS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communications-evidence-export-source-api': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-communications-export-api': [],
  'hermes-communications-export-core': [],
  'hermes-communications-export-persistence': [
    { name: 'hermes-communications-export-core', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-communications-export-runtime': [
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communications-evidence-export-source-api', kind: 'normal' },
    { name: 'hermes-communications-export-api', kind: 'normal' },
    { name: 'hermes-communications-export-core', kind: 'normal' },
    { name: 'hermes-communications-export-persistence', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-managed-vault-client', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
  'hermes-communications-export-assembly': [
    { name: 'hermes-communications-export-persistence', kind: 'normal' },
    { name: 'hermes-communications-export-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-communications-runtime': [
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communications-attachment-contract', kind: 'normal' },
    { name: 'hermes-communications-content-api', kind: 'normal' },
    { name: 'hermes-communications-evidence-export-source-api', kind: 'normal' },
    { name: 'hermes-communications-ingress', kind: 'normal' },
    { name: 'hermes-communications-api', kind: 'normal' },
    { name: 'hermes-communications-domain', kind: 'normal' },
    { name: 'hermes-communications-persistence', kind: 'normal' },
    { name: 'hermes-communications-saved-query-api', kind: 'normal' },
    { name: 'hermes-communications-sender-insights-api', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-managed-vault-client', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
};

const PROTOCOL_THIRD_PARTY_DEPENDENCIES = [
  {
    name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [],
  },
  {
    name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [],
  },
  {
    name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [],
  },
  {
    name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [],
  },
];

const RECOVERY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  'hermes-events-protocol': [
    ...PROTOCOL_THIRD_PARTY_DEPENDENCIES,
    { name: 'hpke', kind: 'normal', source: 'crates_io', version: '=0.14.0', defaultFeatures: false, features: ['alloc', 'chacha', 'getrandom', 'x25519'] },
    { name: 'nats-jwt', kind: 'normal', source: 'crates_io', version: '=0.3.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-runtime-protocol': [
    ...PROTOCOL_THIRD_PARTY_DEPENDENCIES,
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
  ],
  'hermes-gateway-protocol': PROTOCOL_THIRD_PARTY_DEPENDENCIES,
  'hermes-kernel-control-store': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-kernel-control-store-sqlite': [
    {
      name: 'rusqlite', kind: 'normal', source: 'crates_io', version: '=0.32.0', defaultFeatures: false, features: ['backup', 'bundled'],
    },
    {
      name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [],
    },
  ],
  'hermes-kernel': [
    {
      name: 'clap', kind: 'normal', source: 'crates_io', version: '=4.6.2', defaultFeatures: false, features: ['derive', 'error-context', 'help', 'std', 'usage'],
    },
    {
      name: 'directories', kind: 'normal', source: 'crates_io', version: '=6.0.0', defaultFeatures: true, features: [],
    },
    {
      name: 'p256', kind: 'normal', source: 'crates_io', version: '=0.14.0', defaultFeatures: false, features: ['ecdsa'],
    },
    {
      name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [],
    },
    {
      name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [],
    },
    {
      name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [],
    },
    {
      name: 'rcgen', kind: 'normal', source: 'crates_io', version: '=0.13.2', defaultFeatures: true, features: [],
    },
    {
      name: 'rustls', kind: 'normal', source: 'crates_io', version: '=0.23.37', defaultFeatures: false, features: ['ring', 'std'],
    },
    {
      name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [],
    },
    {
      name: 'signal-hook', kind: 'normal', source: 'crates_io', version: '=0.3.18', defaultFeatures: true, features: [],
    },
  ],
  'hermes-secure-file': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
  ],
};

const VAULT_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...RECOVERY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-vault-protocol': [
    { name: 'hpke', kind: 'normal', source: 'crates_io', version: '=0.14.0', defaultFeatures: false, features: ['alloc', 'chacha', 'getrandom', 'x25519'] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-managed-vault-client': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-vault-key-provider': [],
  'hermes-vault-key-provider-file': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
  ],
  'hermes-vault-store-sqlcipher': [
    { name: 'bip39', kind: 'normal', source: 'crates_io', version: '=2.2.2', defaultFeatures: false, features: ['std'] },
    { name: 'chacha20poly1305', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: ['alloc', 'zeroize'] },
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'hkdf', kind: 'normal', source: 'crates_io', version: '=0.13.0', defaultFeatures: true, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'rusqlite', kind: 'normal', source: 'crates_io', version: '=0.32.0', defaultFeatures: false, features: ['backup', 'bundled-sqlcipher'] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-vault-runtime': [
    { name: 'clap', kind: 'normal', source: 'crates_io', version: '=4.6.2', defaultFeatures: false, features: ['derive', 'error-context', 'help', 'std', 'usage'] },
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'hpke', kind: 'normal', source: 'crates_io', version: '=0.14.0', defaultFeatures: false, features: ['alloc', 'chacha', 'getrandom', 'x25519'] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'p256', kind: 'normal', source: 'crates_io', version: '=0.14.0', defaultFeatures: false, features: ['ecdsa'] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const CLOCK_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...VAULT_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-clock-protocol': [],
  'hermes-clock-runtime': [],
};

const TELEMETRY_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...CLOCK_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-telemetry-protocol': [],
  'hermes-telemetry-collector': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
  ],
};

const STORAGE_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...TELEMETRY_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-storage-protocol': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
  ],
  'hermes-storage-control': [],
  'hermes-storage-vault': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-storage-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['net', 'rt', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-storage-postgres': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-storage-pgbouncer': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt', 'time'] },
    { name: 'tokio-postgres', kind: 'normal', source: 'crates_io', version: '=0.7.18', defaultFeatures: false, features: ['runtime'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-storage-migrations': [
    { name: 'pg_query', kind: 'normal', source: 'crates_io', version: '=6.1.1', defaultFeatures: true, features: [] },
  ],
};

const NATS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...STORAGE_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-events-jetstream': [
    { name: 'async-nats', kind: 'normal', source: 'crates_io', version: '=0.49.1', defaultFeatures: true, features: [] },
    { name: 'base64', kind: 'normal', source: 'crates_io', version: '=0.22.1', defaultFeatures: true, features: [] },
    { name: 'futures-util', kind: 'normal', source: 'crates_io', version: '=0.3.32', defaultFeatures: true, features: [] },
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'hpke', kind: 'normal', source: 'crates_io', version: '=0.14.0', defaultFeatures: false, features: ['alloc', 'chacha', 'getrandom', 'x25519'] },
    { name: 'nats-jwt', kind: 'normal', source: 'crates_io', version: '=0.3.0', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-events-authority': [
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-events-authority-runtime-control': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['net', 'rt', 'time'] },
  ],
  'hermes-events-authority-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
  ],
};

const BLOB_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...NATS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-blob-protocol': [],
};

const BLOB_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...BLOB_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-blob-client-contract': [],
  'hermes-blob-client': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
  ],
  'hermes-blob-runtime': [
    { name: 'chacha20poly1305', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: ['alloc', 'zeroize'] },
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-blob-service': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'p256', kind: 'normal', source: 'crates_io', version: '=0.14.0', defaultFeatures: false, features: ['ecdsa'] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const SCHEDULER_PROTOCOL_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...BLOB_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-scheduler-protocol': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
  ],
};

const SCHEDULER_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_PROTOCOL_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-scheduler': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
  ],
};

const SCHEDULER_PERSISTENCE_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-scheduler-persistence': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const GATEWAY_SESSION_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_PERSISTENCE_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-gateway-session-contract': [],
  'hermes-gateway-session': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'p256', kind: 'normal', source: 'crates_io', version: '=0.14.0', defaultFeatures: false, features: ['ecdsa'] },
    { name: 'serde_cbor_2', kind: 'normal', source: 'crates_io', version: '=0.13.0', defaultFeatures: true, features: [] },
    { name: 'url', kind: 'normal', source: 'crates_io', version: '=2.5.8', defaultFeatures: true, features: [] },
    { name: 'webauthn-rs-core', kind: 'normal', source: 'crates_io', version: '=0.5.5', defaultFeatures: true, features: [] },
  ],
};

const SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...GATEWAY_SESSION_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
};

const SCHEDULER_JETSTREAM_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-scheduler-jetstream': [
    { name: 'async-nats', kind: 'normal', source: 'crates_io', version: '=0.49.1', defaultFeatures: true, features: [] },
    { name: 'futures-util', kind: 'normal', source: 'crates_io', version: '=0.3.32', defaultFeatures: true, features: [] },
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'nats-jwt', kind: 'normal', source: 'crates_io', version: '=0.3.0', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['time'] },
  ],
};

const SCHEDULER_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_JETSTREAM_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-scheduler-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['net', 'rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const GATEWAY_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-gateway-protocol': PROTOCOL_THIRD_PARTY_DEPENDENCIES,
  'hermes-kernel': [
    ...SCHEDULER_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST['hermes-kernel'],
    { name: 'chacha20poly1305', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: ['alloc', 'zeroize'] },
    { name: 'quinn', kind: 'normal', source: 'crates_io', version: '=0.11.7', defaultFeatures: true, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['net', 'rt-multi-thread', 'sync', 'time'] },
    { name: 'tokio-rustls', kind: 'normal', source: 'crates_io', version: '=0.26.4', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-gateway-runtime': [
    { name: 'base64', kind: 'normal', source: 'crates_io', version: '=0.22.1', defaultFeatures: true, features: [] },
    { name: 'bytes', kind: 'normal', source: 'crates_io', version: '=1.12.1', defaultFeatures: true, features: [] },
    { name: 'futures-util', kind: 'normal', source: 'crates_io', version: '=0.3.32', defaultFeatures: true, features: [] },
    { name: 'h3', kind: 'normal', source: 'crates_io', version: '=0.0.8', defaultFeatures: true, features: [] },
    { name: 'h3-quinn', kind: 'normal', source: 'crates_io', version: '=0.0.10', defaultFeatures: true, features: [] },
    { name: 'http-body-util', kind: 'normal', source: 'crates_io', version: '=0.1.3', defaultFeatures: true, features: [] },
    { name: 'hyper', kind: 'normal', source: 'crates_io', version: '=1.10.1', defaultFeatures: false, features: ['http1', 'http2', 'server'] },
    { name: 'hyper-util', kind: 'normal', source: 'crates_io', version: '=0.1.20', defaultFeatures: false, features: ['tokio'] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'quinn', kind: 'normal', source: 'crates_io', version: '=0.11.7', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: true, features: ['derive'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['io-util', 'macros', 'net', 'rt', 'sync'] },
    { name: 'tokio-rustls', kind: 'normal', source: 'crates_io', version: '=0.26.4', defaultFeatures: true, features: [] },
    { name: 'webauthn-rs-core', kind: 'normal', source: 'crates_io', version: '=0.5.5', defaultFeatures: true, features: [] },
  ],
};

const MAIL_COMMUNICATIONS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...GATEWAY_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-blob-client-contract': [],
  'hermes-blob-client': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.3.4', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.10.9', defaultFeatures: true, features: [] },
  ],
  'hermes-mail-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
  ],
  'hermes-mail-core': [
    { name: 'base64', kind: 'normal', source: 'crates_io', version: '=0.22.1', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-mail-imap': [
    { name: 'async-imap', kind: 'normal', source: 'crates_io', version: '=0.11.2', defaultFeatures: true, features: [] },
    { name: 'async-native-tls', kind: 'normal', source: 'crates_io', version: '=0.6.0', defaultFeatures: true, features: [] },
    { name: 'async-std', kind: 'normal', source: 'crates_io', version: '=1.13.2', defaultFeatures: true, features: [] },
    { name: 'futures-util', kind: 'normal', source: 'crates_io', version: '=0.3.32', defaultFeatures: true, features: [] },
    { name: 'imap-proto', kind: 'normal', source: 'crates_io', version: '=0.16.7', defaultFeatures: true, features: [] },
  ],
  'hermes-mail-gmail': [
    { name: 'async-native-tls', kind: 'normal', source: 'crates_io', version: '=0.6.0', defaultFeatures: true, features: [] },
    { name: 'async-std', kind: 'normal', source: 'crates_io', version: '=1.13.2', defaultFeatures: true, features: [] },
    { name: 'base64', kind: 'normal', source: 'crates_io', version: '=0.22.1', defaultFeatures: true, features: [] },
    { name: 'futures-util', kind: 'normal', source: 'crates_io', version: '=0.3.32', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: true, features: ['derive'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'hermes-mail-smtp': [
    { name: 'async-native-tls', kind: 'normal', source: 'crates_io', version: '=0.6.0', defaultFeatures: true, features: [] },
    { name: 'async-std', kind: 'normal', source: 'crates_io', version: '=1.13.2', defaultFeatures: true, features: [] },
  ],
  'hermes-mail-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-mail-runtime': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-mail-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'hermes-telegram-api': [
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive'] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
  ],
  'hermes-telegram-core': [
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.10.9', defaultFeatures: true, features: [] },
  ],
  'hermes-telegram-tdlib': [
    { name: 'base64', kind: 'normal', source: 'crates_io', version: '=0.22.1', defaultFeatures: true, features: [] },
    { name: 'libloading', kind: 'normal', source: 'crates_io', version: '=0.8.9', defaultFeatures: true, features: [] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-telegram-persistence': [
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.10.9', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['json', 'postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'hermes-telegram-runtime': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.10.9', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt', 'rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-telegram-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'hermes-whatsapp-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['alloc', 'derive'] },
  ],
  'hermes-whatsapp-core': [],
  'hermes-whatsapp-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'hermes-whatsapp-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread'] },
  ],
  'hermes-whatsapp-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'hermes-zulip-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
  ],
  'hermes-zulip-core': [],
  'hermes-zulip-http': [
    { name: 'async-native-tls', kind: 'normal', source: 'crates_io', version: '=0.6.0', defaultFeatures: true, features: [] },
    { name: 'async-std', kind: 'normal', source: 'crates_io', version: '=1.13.2', defaultFeatures: true, features: [] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-zulip-persistence': [{ name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] }],
  'hermes-zulip-runtime': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-communications-ingress': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communications-attachment-contract': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communications-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communications-domain': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communications-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'hermes-communications-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt', 'rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-communications-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const FIRST_OWNER_THIRD_PARTY_DEPENDENCY_ALLOWLIST = Object.fromEntries(
  FIRST_OWNER_PRODUCTION_PACKAGES.map(({ name }) => [
    name,
    MAIL_COMMUNICATIONS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST[name],
  ]),
);

const ATTACHMENT_SECURITY_ENGINE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...FIRST_OWNER_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-attachment-security-contract': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-attachment-security-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-attachment-security-clamav': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-attachment-security-persistence': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'hermes-attachment-security-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-attachment-security-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const MAIL_OUTBOUND_MIME_ATTACHMENTS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_SECURITY_ENGINE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  ...Object.fromEntries(
    MAIL_OUTBOUND_MIME_ATTACHMENTS_PRODUCTION_PACKAGES
      .filter(({ owner }) => owner === 'mail')
      .map(({ name }) => [
        name,
        MAIL_COMMUNICATIONS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST[name],
      ]),
  ),
};

const COMMUNICATIONS_CONTENT_READ_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...MAIL_OUTBOUND_MIME_ATTACHMENTS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communications-content-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communications-runtime': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: true, features: [] },
    ...MAIL_COMMUNICATIONS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST[
      'hermes-communications-runtime'
    ],
  ],
};

const COMMUNICATIONS_SAVED_SEARCH_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CONTENT_READ_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communications-saved-query-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATIONS_SENDER_INSIGHTS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_SAVED_SEARCH_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communications-sender-insights-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_EXPORT_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delivery-intent-api': [],
  'hermes-communication-delivery-intent-core': [
    { name: 'hermes-communications-api', kind: 'normal' },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delivery-intent-persistence': [
    { name: 'hermes-communication-delivery-intent-core', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delivery-intent-runtime': [
    { name: 'hermes-communication-delivery-intent-api', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-core', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-event-adapters', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-persistence', kind: 'normal' },
    { name: 'hermes-communications-api', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delivery-intent-assembly': [
    { name: 'hermes-communication-delivery-intent-persistence', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-mail-delivery-intent-contract': [
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-telegram-delivery-intent-contract': [
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-whatsapp-delivery-intent-contract': [
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-zulip-delivery-intent-contract': [
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-communication-delivery-intent-event-adapters': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-mail-delivery-intent-contract', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-telegram-delivery-intent-contract', kind: 'normal' },
    { name: 'hermes-whatsapp-delivery-intent-contract', kind: 'normal' },
    { name: 'hermes-zulip-delivery-intent-contract', kind: 'normal' },
  ],
};

const DELIVERY_INTENT_TARGET_BOUND_BLOB_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-mail-runtime': [
    { name: 'hermes-mail-api', kind: 'normal' },
    { name: 'hermes-mail-core', kind: 'normal' },
    { name: 'hermes-mail-imap', kind: 'normal' },
    { name: 'hermes-mail-gmail', kind: 'normal' },
    { name: 'hermes-mail-smtp', kind: 'normal' },
    { name: 'hermes-mail-persistence', kind: 'normal' },
    { name: 'hermes-mail-delivery-intent-contract', kind: 'normal' },
    { name: 'hermes-attachment-security-contract', kind: 'normal' },
    { name: 'hermes-communications-attachment-contract', kind: 'normal' },
    { name: 'hermes-communications-ingress', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-managed-vault-client', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
    { name: 'hermes-vault-protocol', kind: 'normal' },
  ],
  'hermes-communication-delivery-intent-runtime': [
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-api', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-core', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-event-adapters', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-persistence', kind: 'normal' },
    { name: 'hermes-communications-api', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-mail-delivery-intent-contract', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
    { name: 'hermes-telegram-delivery-intent-contract', kind: 'normal' },
    { name: 'hermes-whatsapp-delivery-intent-contract', kind: 'normal' },
    { name: 'hermes-zulip-delivery-intent-contract', kind: 'normal' },
  ],
};

const COMMUNICATION_BULK_ACTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...DELIVERY_INTENT_TARGET_BOUND_BLOB_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-bulk-action-api': [],
  'hermes-communication-bulk-action-core': [],
};

const COMMUNICATION_BULK_ACTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_BULK_ACTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-bulk-action-persistence': [
    { name: 'hermes-communication-bulk-action-core', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_BULK_ACTION_RUNTIME_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_BULK_ACTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-bulk-action-runtime': [
    { name: 'hermes-communication-bulk-action-api', kind: 'normal' },
    { name: 'hermes-communication-bulk-action-core', kind: 'normal' },
    { name: 'hermes-communication-bulk-action-persistence', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-api', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATION_BULK_ACTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_BULK_ACTION_RUNTIME_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-bulk-action-assembly': [
    { name: 'hermes-communication-bulk-action-persistence', kind: 'normal' },
    { name: 'hermes-communication-bulk-action-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_BULK_ACTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delayed-delivery-api': [],
  'hermes-communication-delayed-delivery-core': [],
};

const COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delayed-delivery-persistence': [
    { name: 'hermes-communication-delayed-delivery-core', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_EXECUTION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delayed-delivery-execution': [
    { name: 'hermes-communication-delivery-intent-api', kind: 'normal' },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_EVENT_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_EXECUTION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delayed-delivery-event-adapters': [
    { name: 'hermes-communication-delayed-delivery-api', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-scheduler-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_EVENT_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delayed-delivery-runtime-adapters': [
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communication-delayed-delivery-api', kind: 'normal' },
    { name: 'hermes-communication-delayed-delivery-execution', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-api', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delayed-delivery-store-adapters': [
    { name: 'hermes-communication-delayed-delivery-execution', kind: 'normal' },
    { name: 'hermes-communication-delayed-delivery-persistence', kind: 'normal' },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delayed-delivery-runtime': [
    { name: 'hermes-communication-delayed-delivery-api', kind: 'normal' },
    { name: 'hermes-communication-delayed-delivery-core', kind: 'normal' },
    { name: 'hermes-communication-delayed-delivery-event-adapters', kind: 'normal' },
    { name: 'hermes-communication-delayed-delivery-execution', kind: 'normal' },
    { name: 'hermes-communication-delayed-delivery-persistence', kind: 'normal' },
    { name: 'hermes-communication-delayed-delivery-runtime-adapters', kind: 'normal' },
    { name: 'hermes-communication-delayed-delivery-store-adapters', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-api', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-scheduler-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delayed-delivery-assembly': [
    { name: 'hermes-communication-delayed-delivery-persistence', kind: 'normal' },
    { name: 'hermes-communication-delayed-delivery-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-cross-channel-forward-api': [],
  'hermes-communication-cross-channel-forward-core': [],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-cross-channel-forward-persistence': [
    { name: 'hermes-communication-cross-channel-forward-core', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communications-runtime':
    COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST[
      'hermes-communications-runtime'
    ].flatMap((dependency) => (
      dependency.name === 'hermes-communications-evidence-export-source-api'
        ? [
            dependency,
            {
              name: 'hermes-communications-cross-channel-forward-source-api',
              kind: 'normal',
            },
          ]
        : [dependency]
    )),
  'hermes-communications-cross-channel-forward-source-api': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delivery-intent-ingress-api': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-cross-channel-forward-persistence': [
    { name: 'hermes-communication-cross-channel-forward-core', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-cross-channel-forward-runtime': [
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communication-cross-channel-forward-api', kind: 'normal' },
    { name: 'hermes-communication-cross-channel-forward-core', kind: 'normal' },
    { name: 'hermes-communication-cross-channel-forward-persistence', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-ingress-api', kind: 'normal' },
    { name: 'hermes-communications-cross-channel-forward-source-api', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delivery-intent-runtime': [
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-api', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-core', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-event-adapters', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-ingress-api', kind: 'normal' },
    { name: 'hermes-communication-delivery-intent-persistence', kind: 'normal' },
    { name: 'hermes-communications-api', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-mail-delivery-intent-contract', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
    { name: 'hermes-telegram-delivery-intent-contract', kind: 'normal' },
    { name: 'hermes-whatsapp-delivery-intent-contract', kind: 'normal' },
    { name: 'hermes-zulip-delivery-intent-contract', kind: 'normal' },
  ],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-cross-channel-forward-assembly': [
    { name: 'hermes-communication-cross-channel-forward-persistence', kind: 'normal' },
    { name: 'hermes-communication-cross-channel-forward-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communications-call-evidence-ingress': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-communications-call-evidence-core': [
    { name: 'hermes-communications-call-evidence-ingress', kind: 'normal' },
  ],
};

const COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communications-call-evidence-persistence': [
    { name: 'hermes-communications-call-evidence-core', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATIONS_CALL_EVIDENCE_MANAGED_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communications-runtime':
    COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST[
      'hermes-communications-runtime'
    ].flatMap((dependency) => (
      dependency.name === 'hermes-communications-attachment-contract'
        ? [
            dependency,
            { name: 'hermes-communications-call-evidence-core', kind: 'normal' },
            { name: 'hermes-communications-call-evidence-ingress', kind: 'normal' },
            { name: 'hermes-communications-call-evidence-persistence', kind: 'normal' },
          ]
        : [dependency]
    )),
  'hermes-communications-assembly':
    COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST[
      'hermes-communications-assembly'
    ].filter((dependency) => dependency.name !== 'hermes-communications-persistence'),
};

const COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CALL_EVIDENCE_MANAGED_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communications-call-evidence-api': [],
  'hermes-communications-runtime':
    COMMUNICATIONS_CALL_EVIDENCE_MANAGED_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST[
      'hermes-communications-runtime'
    ].flatMap((dependency) => (
      dependency.name === 'hermes-communications-call-evidence-core'
        ? [
            { name: 'hermes-communications-call-evidence-api', kind: 'normal' },
            dependency,
          ]
        : [dependency]
    )),
};

const REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-review-attention-api': [],
  'hermes-review-attention-core': [],
};

const REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-review-attention-persistence': [
    { name: 'hermes-review-attention-core', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-review-attention-runtime': [
    { name: 'hermes-review-attention-api', kind: 'normal' },
    { name: 'hermes-review-attention-core', kind: 'normal' },
    { name: 'hermes-review-attention-persistence', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
};

const REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-review-attention-assembly': [
    { name: 'hermes-review-attention-persistence', kind: 'normal' },
    { name: 'hermes-review-attention-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATIONS_AI_SOURCE_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communications-runtime':
    REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST[
      'hermes-communications-runtime'
    ].flatMap((dependency) => (
      dependency.name === 'hermes-communications-attachment-contract'
        ? [
            { name: 'hermes-communications-ai-source-api', kind: 'normal' },
            dependency,
          ]
        : [dependency]
    )),
  'hermes-communications-ai-source-api': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-communication-reply-suggestion-api': [],
  'hermes-communication-reply-suggestion-core': [],
  'hermes-communication-reply-suggestion-persistence': [
    { name: 'hermes-communication-reply-suggestion-core', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-communication-reply-suggestion-runtime': [
    { name: 'hermes-ai-contracts', kind: 'normal' },
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communication-reply-suggestion-api', kind: 'normal' },
    { name: 'hermes-communication-reply-suggestion-core', kind: 'normal' },
    { name: 'hermes-communication-reply-suggestion-persistence', kind: 'normal' },
    { name: 'hermes-communications-ai-source-api', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
  'hermes-communication-reply-suggestion-assembly': [
    { name: 'hermes-communication-reply-suggestion-persistence', kind: 'normal' },
    { name: 'hermes-communication-reply-suggestion-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-ai-contracts': [
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-ai-inference-core': [
    { name: 'hermes-ai-contracts', kind: 'normal' },
  ],
  'hermes-ai-inference-persistence': [
    { name: 'hermes-ai-contracts', kind: 'normal' },
    { name: 'hermes-ai-inference-core', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-ollama-ai-api': [
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-ollama-ai-assembly': [
    { name: 'hermes-ollama-ai-api', kind: 'normal' },
    { name: 'hermes-ollama-ai-persistence', kind: 'normal' },
    { name: 'hermes-ollama-ai-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-ollama-ai-core': [
    { name: 'hermes-ai-contracts', kind: 'normal' },
    { name: 'hermes-ollama-ai-api', kind: 'normal' },
  ],
  'hermes-ollama-ai-http': [
    { name: 'hermes-ollama-ai-api', kind: 'normal' },
    { name: 'hermes-ollama-ai-core', kind: 'normal' },
  ],
  'hermes-ollama-ai-persistence': [
    { name: 'hermes-ai-contracts', kind: 'normal' },
    { name: 'hermes-ollama-ai-core', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-ollama-ai-runtime': [
    { name: 'hermes-ai-contracts', kind: 'normal' },
    { name: 'hermes-ollama-ai-api', kind: 'normal' },
    { name: 'hermes-ollama-ai-core', kind: 'normal' },
    { name: 'hermes-ollama-ai-http', kind: 'normal' },
    { name: 'hermes-ollama-ai-persistence', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
};

const ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_AI_SOURCE_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-attachment-archive-inspection-api': [],
  'hermes-attachment-archive-inspection-ingress': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
  'hermes-attachment-archive-inspection-core': [
    { name: 'hermes-attachment-archive-inspection-api', kind: 'normal' },
  ],
  'hermes-attachment-archive-inspection-zip': [
    { name: 'hermes-attachment-archive-inspection-core', kind: 'normal' },
  ],
};

const ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-attachment-archive-inspection-persistence': [
    { name: 'hermes-attachment-archive-inspection-core', kind: 'normal' },
    { name: 'hermes-attachment-archive-inspection-ingress', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-attachment-archive-inspection-runtime': [
    { name: 'hermes-attachment-archive-inspection-api', kind: 'normal' },
    { name: 'hermes-attachment-archive-inspection-core', kind: 'normal' },
    { name: 'hermes-attachment-archive-inspection-ingress', kind: 'normal' },
    { name: 'hermes-attachment-archive-inspection-persistence', kind: 'normal' },
    { name: 'hermes-attachment-archive-inspection-zip', kind: 'normal' },
    { name: 'hermes-attachment-security-contract', kind: 'normal' },
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communications-attachment-contract', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
};

const ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-attachment-archive-inspection-assembly': [
    { name: 'hermes-attachment-archive-inspection-api', kind: 'normal' },
    { name: 'hermes-attachment-archive-inspection-persistence', kind: 'normal' },
    { name: 'hermes-attachment-archive-inspection-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_SUMMARY_BUILD_UNITS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-summary-api': [],
  'hermes-communication-summary-core': [],
  'hermes-communication-summary-persistence': [
    { name: 'hermes-communication-summary-core', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-communication-summary-runtime': [
    { name: 'hermes-ai-contracts', kind: 'normal' },
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communication-summary-api', kind: 'normal' },
    { name: 'hermes-communication-summary-core', kind: 'normal' },
    { name: 'hermes-communication-summary-persistence', kind: 'normal' },
    { name: 'hermes-communications-ai-source-api', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
  'hermes-communication-summary-assembly': [
    { name: 'hermes-communication-summary-persistence', kind: 'normal' },
    { name: 'hermes-communication-summary-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_TRANSLATION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_SUMMARY_BUILD_UNITS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-translation-api': [],
  'hermes-communication-translation-core': [],
};

const COMMUNICATION_TRANSLATION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TRANSLATION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-translation-persistence': [
    { name: 'hermes-communication-translation-core', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_TRANSLATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TRANSLATION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-translation-runtime': [
    { name: 'hermes-ai-contracts', kind: 'normal' },
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communication-translation-api', kind: 'normal' },
    { name: 'hermes-communication-translation-core', kind: 'normal' },
    { name: 'hermes-communication-translation-persistence', kind: 'normal' },
    { name: 'hermes-communications-ai-source-api', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATION_TRANSLATION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TRANSLATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-translation-assembly': [
    { name: 'hermes-communication-translation-persistence', kind: 'normal' },
    { name: 'hermes-communication-translation-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_EXPLANATION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TRANSLATION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-explanation-api': [],
  'hermes-communication-explanation-core': [],
};

const COMMUNICATION_EXPLANATION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_EXPLANATION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-explanation-persistence': [
    { name: 'hermes-communication-explanation-core', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_EXPLANATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_EXPLANATION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-explanation-runtime': [
    { name: 'hermes-ai-contracts', kind: 'normal' },
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communication-explanation-api', kind: 'normal' },
    { name: 'hermes-communication-explanation-core', kind: 'normal' },
    { name: 'hermes-communication-explanation-persistence', kind: 'normal' },
    { name: 'hermes-communications-ai-source-api', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATION_EXPLANATION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_EXPLANATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-explanation-assembly': [
    { name: 'hermes-communication-explanation-persistence', kind: 'normal' },
    { name: 'hermes-communication-explanation-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_EXPLANATION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-recipient-suggestion-api': [],
  'hermes-communication-recipient-suggestion-core': [],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communications-recipient-source-api': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-recipient-suggestion-persistence': [
    { name: 'hermes-communication-recipient-suggestion-core', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-communications-recipient-source-api': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-recipient-suggestion-persistence': [
    { name: 'hermes-communication-recipient-suggestion-core', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-communication-recipient-suggestion-runtime': [
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communication-recipient-suggestion-api', kind: 'normal' },
    { name: 'hermes-communication-recipient-suggestion-core', kind: 'normal' },
    { name: 'hermes-communication-recipient-suggestion-persistence', kind: 'normal' },
    { name: 'hermes-communications-recipient-source-api', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
  'hermes-communications-recipient-source-api': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communications-runtime': [
    ...COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST['hermes-communications-runtime'],
    { name: 'hermes-communications-recipient-source-api', kind: 'normal' },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-recipient-suggestion-assembly': [
    { name: 'hermes-communication-recipient-suggestion-persistence', kind: 'normal' },
    { name: 'hermes-communication-recipient-suggestion-runtime', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-task-candidate-api': [],
  'hermes-communication-task-candidate-core': [],
  'hermes-communications-task-source-api': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-task-candidate-persistence': [
    { name: 'hermes-communication-task-candidate-core', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_TASK_CANDIDATE_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communication-task-candidate-runtime': [
    { name: 'hermes-blob-client', kind: 'normal' },
    { name: 'hermes-communication-task-candidate-api', kind: 'normal' },
    { name: 'hermes-communication-task-candidate-core', kind: 'normal' },
    { name: 'hermes-communication-task-candidate-persistence', kind: 'normal' },
    { name: 'hermes-communications-task-source-api', kind: 'normal' },
    { name: 'hermes-events-jetstream', kind: 'normal' },
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-runtime-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
    { name: 'hermes-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATION_TASK_CANDIDATE_SOURCE_PRODUCER_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TASK_CANDIDATE_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'hermes-communications-runtime': [
    ...COMMUNICATION_TASK_CANDIDATE_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST['hermes-communications-runtime'],
    { name: 'hermes-communications-task-source-api', kind: 'normal' },
  ],
};

const COMMUNICATIONS_EXPORT_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_SENDER_INSIGHTS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communications-evidence-export-source-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communications-export-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communications-export-core': [
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communications-export-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'hermes-communications-export-runtime': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: true, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt', 'rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-communications-export-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_EXPORT_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delivery-intent-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communication-delivery-intent-core': [],
};

const COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delivery-intent-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delivery-intent-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt', 'rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delivery-intent-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-mail-delivery-intent-contract': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-telegram-delivery-intent-contract': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-whatsapp-delivery-intent-contract': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-zulip-delivery-intent-contract': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communication-delivery-intent-event-adapters': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const DELIVERY_INTENT_TARGET_BOUND_BLOB_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-mail-runtime': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_BULK_ACTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...DELIVERY_INTENT_TARGET_BOUND_BLOB_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-bulk-action-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communication-bulk-action-core': [],
};

const COMMUNICATION_BULK_ACTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_BULK_ACTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-bulk-action-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const COMMUNICATION_BULK_ACTION_RUNTIME_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_BULK_ACTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-bulk-action-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_BULK_ACTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_BULK_ACTION_RUNTIME_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-bulk-action-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_BULK_ACTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delayed-delivery-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communication-delayed-delivery-core': [],
};

const COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delayed-delivery-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_EXECUTION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delayed-delivery-execution': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['macros', 'rt'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_EVENT_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_EXECUTION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delayed-delivery-event-adapters': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_EVENT_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delayed-delivery-runtime-adapters': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delayed-delivery-store-adapters': [],
};

const COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delayed-delivery-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delayed-delivery-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-cross-channel-forward-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communication-cross-channel-forward-core': [],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-cross-channel-forward-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communications-cross-channel-forward-source-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-delivery-intent-ingress-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST =
  COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST;

const COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-cross-channel-forward-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_CONSUMER_THIRD_PARTY_DEPENDENCY_ALLOWLIST =
  COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST;

const COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_CONSUMER_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-cross-channel-forward-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communications-call-evidence-ingress': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communications-call-evidence-core': [
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communications-call-evidence-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communications-call-evidence-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-review-attention-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-review-attention-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-review-attention-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-review-attention-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-review-attention-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATIONS_AI_SOURCE_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communications-ai-source-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communication-reply-suggestion-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communication-reply-suggestion-core': [],
  'hermes-communication-reply-suggestion-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'hermes-communication-reply-suggestion-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-communication-reply-suggestion-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'hermes-ai-contracts': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-ai-inference-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-ai-inference-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'hermes-ollama-ai-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
  ],
  'hermes-ollama-ai-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'hermes-ollama-ai-core': [
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-ollama-ai-http': [
    { name: 'async-std', kind: 'normal', source: 'crates_io', version: '=1.13.2', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-ollama-ai-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'hermes-ollama-ai-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_AI_SOURCE_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-attachment-archive-inspection-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-attachment-archive-inspection-ingress': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-attachment-archive-inspection-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-attachment-archive-inspection-zip': [
    { name: 'zip', kind: 'normal', source: 'crates_io', version: '=6.0.0', defaultFeatures: false, features: ['deflate-flate2-zlib-rs'] },
  ],
};

const ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-attachment-archive-inspection-persistence': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-attachment-archive-inspection-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-attachment-archive-inspection-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_SUMMARY_BUILD_UNITS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-summary-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communication-summary-core': [],
  'hermes-communication-summary-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'hermes-communication-summary-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-communication-summary-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_TRANSLATION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_SUMMARY_BUILD_UNITS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-translation-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communication-translation-core': [],
};

const COMMUNICATION_TRANSLATION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TRANSLATION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-translation-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const COMMUNICATION_TRANSLATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TRANSLATION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-translation-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_TRANSLATION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TRANSLATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-translation-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_EXPLANATION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TRANSLATION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-explanation-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communication-explanation-core': [],
};

const COMMUNICATION_EXPLANATION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_EXPLANATION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-explanation-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const COMMUNICATION_EXPLANATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_EXPLANATION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-explanation-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_EXPLANATION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_EXPLANATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-explanation-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_EXPLANATION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-recipient-suggestion-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communication-recipient-suggestion-core': [],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communications-recipient-source-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-recipient-suggestion-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'hermes-communications-recipient-source-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-recipient-suggestion-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'hermes-communication-recipient-suggestion-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'hermes-communications-recipient-source-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-recipient-suggestion-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-task-candidate-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communication-task-candidate-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'hermes-communications-task-source-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-task-candidate-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const COMMUNICATION_TASK_CANDIDATE_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'hermes-communication-task-candidate-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const FORBIDDEN_DEPENDENCIES = [
  'async-nats',
  'nats',
  'sqlx',
  'tokio-postgres',
  'postgres',
  'diesel',
  'sea-orm',
  'deadpool-postgres',
  'bb8-postgres',
  'reqwest',
  'ureq',
  'isahc',
  'surf',
  'awc',
];

const RECOVERY_FORBIDDEN_DEPENDENCY_PREFIXES = [
  'hermes-vault-',
  'hermes-storage-',
  'hermes-integration-',
  'hermes-provider-',
];

const VAULT_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES = [
  'hermes-storage-',
  'hermes-integration-',
  'hermes-provider-',
];

const STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES = [
  'hermes-integration-',
  'hermes-provider-',
];

const KERNEL_PROFILE_KEYS = [
  'maximumState',
  'allowedStates',
  'forbiddenStates',
  'activeComponents',
  'transport',
  'onlineOperations',
  'bootstrapOperations',
  'offlineOperations',
  'externalServices',
  'managedChildren',
  'publicGatewayEnabled',
  'networkListenerEnabled',
  'moduleRegistrationEnabled',
  'managedLaunchEnabled',
  'natsDataPlaneEnabled',
  'businessDataPlaneEnabled',
  'wholeInstanceBackupEnabled',
  'clock',
];

const KERNEL_PROFILE = {
  maximumState: 'recovery_only',
  allowedStates: [
    'cold_start',
    'bootstrap',
    'recovery_only',
    'quiescing',
    'draining',
    'stopped',
    'fatal',
  ],
  forbiddenStates: [
    'infrastructure_starting',
    'modules_starting',
    'ready',
    'degraded',
  ],
  activeComponents: ['supervisor', 'core_gateway'],
  transport: 'local_ipc_only',
  onlineOperations: [
    'status',
    'control_store_validate',
    'control_store_export',
    'shutdown',
  ],
  bootstrapOperations: ['initial_owner_enrollment_inherited_fd'],
  offlineOperations: ['control_store_restore', 'control_store_reset'],
  externalServices: [],
  managedChildren: [],
  networkListenerEnabled: false,
  moduleRegistrationEnabled: false,
  managedLaunchEnabled: false,
};

const MODULE_CONTROL_PROFILE = {
  maximumState: 'module_control_plane',
  allowedStates: ['cold_start', 'bootstrap', 'recovery_only', 'module_control_plane', 'quiescing', 'draining', 'stopped', 'fatal'],
  forbiddenStates: ['infrastructure_starting', 'modules_starting', 'ready', 'degraded'],
  activeComponents: ['supervisor', 'module_registry', 'capability_router', 'core_gateway', 'settings_registry'],
  transport: 'local_ipc_only',
  onlineOperations: ['status', 'control_store_validate', 'control_store_export', 'shutdown', 'module_registration', 'owner_control', 'external_runtime_session'],
  bootstrapOperations: ['initial_owner_enrollment_inherited_fd'],
  offlineOperations: ['control_store_restore', 'control_store_reset'],
  externalServices: [],
  managedChildren: [],
  networkListenerEnabled: false,
  moduleRegistrationEnabled: true,
  managedLaunchEnabled: false,
};

const SERVER_BOOTSTRAP_PAIRING_PROFILE = {
  maximumState: 'module_control_plane',
  allowedStates: ['cold_start', 'bootstrap', 'recovery_only', 'module_control_plane', 'quiescing', 'draining', 'stopped', 'fatal'],
  forbiddenStates: ['infrastructure_starting', 'modules_starting', 'ready', 'degraded'],
  activeComponents: ['supervisor', 'module_registry', 'capability_router', 'core_gateway', 'settings_registry'],
  transport: 'local_ipc_and_one_shot_bootstrap_tls',
  onlineOperations: ['status', 'control_store_validate', 'control_store_export', 'shutdown', 'module_registration', 'owner_control', 'external_runtime_session'],
  bootstrapOperations: ['initial_owner_enrollment_inherited_fd', 'server_bootstrap_pairing'],
  offlineOperations: ['control_store_restore', 'control_store_reset'],
  externalServices: [],
  managedChildren: [],
  networkListenerEnabled: true,
  moduleRegistrationEnabled: true,
  managedLaunchEnabled: false,
};

const MANAGED_LAUNCH_TRUST_PROFILE = {
  maximumState: 'module_control_plane',
  allowedStates: ['cold_start', 'bootstrap', 'recovery_only', 'module_control_plane', 'quiescing', 'draining', 'stopped', 'fatal'],
  forbiddenStates: ['infrastructure_starting', 'modules_starting', 'ready', 'degraded'],
  activeComponents: ['supervisor', 'module_registry', 'capability_router', 'core_gateway', 'settings_registry'],
  transport: 'local_ipc_and_one_shot_bootstrap_tls',
  onlineOperations: ['status', 'control_store_validate', 'control_store_export', 'shutdown', 'module_registration', 'owner_control', 'external_runtime_session'],
  bootstrapOperations: ['initial_owner_enrollment_inherited_fd', 'server_bootstrap_pairing'],
  offlineOperations: ['control_store_restore', 'control_store_reset'],
  externalServices: [],
  managedChildren: ['bundled_native_module_runtime'],
  networkListenerEnabled: true,
  moduleRegistrationEnabled: true,
  managedLaunchEnabled: true,
};

const FIRST_OWNER_PROFILE = {
  ...MANAGED_LAUNCH_TRUST_PROFILE,
  publicGatewayEnabled: true,
  natsDataPlaneEnabled: true,
  businessDataPlaneEnabled: true,
  wholeInstanceBackupEnabled: true,
};

const FIRST_OWNER_INVENTORY = {
  domains: ['communications'],
  integrations: [],
  workflows: [],
  engines: [],
  businessCapabilities: [
    'communications.attachment.blob-admission.observe.v1',
    'communications.attachment.safety-verdict.observe.v1',
    'communications.blob.v1',
    'communications.events.v1',
    'communications.observe.v1',
    'communications.query.v1',
    'communications.search.index.v1',
    'communications.storage.v1',
  ],
};

const ATTACHMENT_SECURITY_ENGINE_INVENTORY = {
  domains: ['communications'],
  integrations: [],
  workflows: [],
  engines: ['attachment_security'],
  businessCapabilities: [
    'attachment_security.blob.v1',
    'attachment_security.candidate.observe.v1',
    'attachment_security.communications-state.observe.v1',
    'attachment_security.storage.v1',
    'attachment_security.verdict.publish.v1',
    ...FIRST_OWNER_INVENTORY.businessCapabilities,
  ],
};

const MAIL_OUTBOUND_MIME_ATTACHMENTS_INVENTORY = {
  domains: ['communications'],
  integrations: ['mail'],
  workflows: [],
  engines: ['attachment_security'],
  businessCapabilities: [
    ...ATTACHMENT_SECURITY_ENGINE_INVENTORY.businessCapabilities,
    'mail.attachment-anchor.consume.v1',
    'mail.attachment-blob-admission.publish.v1',
    'mail.attachment-safety-state.consume.v1',
    'mail.attachment.scan-candidate.publish.v1',
    'mail.blob.v1',
    'mail.communication-observed.publish.v1',
    'mail.delivery.query.v1',
    'mail.delivery.v1',
    'mail.gmail.credentials.v1',
    'mail.gmail.oauth-refresh.credentials.v1',
    'mail.gmail.oauth-setup.credentials.v1',
    'mail.imap.credentials.v1',
    'mail.oauth.complete.v1',
    'mail.oauth.query.v1',
    'mail.oauth.refresh.v1',
    'mail.oauth.start.v1',
    'mail.smtp.credentials.v1',
    'mail.storage.v1',
    'mail.sync.v1',
  ],
};

const COMMUNICATIONS_CONTENT_READ_INVENTORY = {
  ...MAIL_OUTBOUND_MIME_ATTACHMENTS_INVENTORY,
  businessCapabilities: [
    ...MAIL_OUTBOUND_MIME_ATTACHMENTS_INVENTORY.businessCapabilities,
    'communications.content.v1',
  ].sort(),
};

const COMMUNICATIONS_SAVED_SEARCH_INVENTORY = {
  ...COMMUNICATIONS_CONTENT_READ_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATIONS_CONTENT_READ_INVENTORY.businessCapabilities,
    'communications.saved-search.v1',
  ].sort(),
};

const COMMUNICATIONS_SENDER_INSIGHTS_INVENTORY = {
  ...COMMUNICATIONS_SAVED_SEARCH_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATIONS_SAVED_SEARCH_INVENTORY.businessCapabilities,
    'communications.sender-insights.v1',
  ].sort(),
};

const COMMUNICATIONS_EXPORT_INVENTORY = {
  ...COMMUNICATIONS_SENDER_INSIGHTS_INVENTORY,
  workflows: ['communications_export'],
  businessCapabilities: [
    ...COMMUNICATIONS_SENDER_INSIGHTS_INVENTORY.businessCapabilities,
    'communications.export-source.blob.v1',
    'communications.export-source.v1',
    'communications.export.v1',
    'communications_export.blob.v1',
    'communications_export.events.v1',
    'communications_export.storage.v1',
  ].sort(),
};

const COMMUNICATION_DELIVERY_INTENT_INVENTORY = {
  ...COMMUNICATIONS_EXPORT_INVENTORY,
  workflows: [
    'communication_cross_channel_forward',
    'communication_delivery_intent',
    'communications_export',
  ],
  businessCapabilities: [
    ...COMMUNICATIONS_EXPORT_INVENTORY.businessCapabilities,
    'communication.cross_channel_forward.v1',
    'communication_cross_channel_forward.blob.v1',
    'communication_cross_channel_forward.delivery_rejected.v1',
    'communication_cross_channel_forward.delivery_submit.v1',
    'communication_cross_channel_forward.delivery_submitted.v1',
    'communication_cross_channel_forward.source_prepare.v1',
    'communication_cross_channel_forward.source_prepared.v1',
    'communication_cross_channel_forward.source_rejected.v1',
    'communication_cross_channel_forward.storage.v1',
    'communication_delivery_intent.blob.v1',
    'communication_delivery_intent.ingress_rejected.v1',
    'communication_delivery_intent.ingress_submit.v1',
    'communication_delivery_intent.ingress_submitted.v1',
    'communication_delivery_intent.mail.events.v1',
    'communication_delivery_intent.storage.v1',
    'communication_delivery_intent.telegram.events.v1',
    'communication_delivery_intent.whatsapp.events.v1',
    'communication_delivery_intent.zulip.events.v1',
    'communications.cross-channel-forward-source.blob.v1',
    'communications.cross-channel-forward-source.v1',
  ].sort(),
};

const REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_INVENTORY = {
  ...COMMUNICATION_DELIVERY_INTENT_INVENTORY,
  domains: ['communications', 'review'],
  businessCapabilities: [
    ...COMMUNICATION_DELIVERY_INTENT_INVENTORY.businessCapabilities,
    'review.communication-attention.command.v1',
    'review.communication-attention.query.v1',
    'review.communication-attention.realtime.v1',
  ].sort(),
};

const REVIEW_COMMUNICATIONS_ATTENTION_LIVE_INVENTORY = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'review.communication-attention.storage.v1',
  ].sort(),
};

const COMMUNICATIONS_AI_SOURCE_CONTRACT_INVENTORY = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_LIVE_INVENTORY,
  workflows: [
    ...REVIEW_COMMUNICATIONS_ATTENTION_LIVE_INVENTORY.workflows,
    'communication_reply_suggestion',
  ].sort(),
  engines: [
    ...REVIEW_COMMUNICATIONS_ATTENTION_LIVE_INVENTORY.engines,
    'ai',
  ].sort(),
  businessCapabilities: [
    ...REVIEW_COMMUNICATIONS_ATTENTION_LIVE_INVENTORY.businessCapabilities,
    'communications.ai-reply-source.blob.v1',
    'communications.ai-reply-source.v1',
    'communications.ai-summary-source.blob.v1',
    'communications.ai-summary-source.v1',
  ].sort(),
};

const ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_INVENTORY = {
  ...COMMUNICATIONS_AI_SOURCE_CONTRACT_INVENTORY,
  engines: [
    ...COMMUNICATIONS_AI_SOURCE_CONTRACT_INVENTORY.engines,
    'attachment_archive_inspection',
  ].sort(),
  businessCapabilities: [
    ...COMMUNICATIONS_AI_SOURCE_CONTRACT_INVENTORY.businessCapabilities,
    'attachment_security.archive-delegation-result.publish.v1',
    'attachment_security.archive-inspection-delegation.v1',
  ].sort(),
};

const ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_INVENTORY = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'attachment_archive_inspection.blob.v1',
    'attachment_archive_inspection.candidate.observe.v1',
    'attachment_archive_inspection.custody-request.publish.v1',
    'attachment_archive_inspection.custody-result.consume.v1',
    'attachment_archive_inspection.safety-state.observe.v1',
    'attachment_archive_inspection.storage.v1',
  ].sort(),
};

const ATTACHMENT_ARCHIVE_INSPECTION_CLIENT_INVENTORY = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_INVENTORY,
  businessCapabilities: [
    ...ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_INVENTORY.businessCapabilities,
    'attachment.archive_inspection.v1',
  ].sort(),
};

const COMMUNICATION_SUMMARY_BUILD_UNITS_INVENTORY = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_CLIENT_INVENTORY,
  workflows: [
    ...ATTACHMENT_ARCHIVE_INSPECTION_CLIENT_INVENTORY.workflows,
    'communication_summary',
  ].sort(),
  businessCapabilities: [
    ...ATTACHMENT_ARCHIVE_INSPECTION_CLIENT_INVENTORY.businessCapabilities,
    'communication.summary.v1',
    'communication_summary.inference.v1',
    'communication_summary.source.blob.v1',
    'communication_summary.source_prepare.v1',
    'communication_summary.source_prepared.v1',
    'communication_summary.source_rejected.v1',
    'communication_summary.storage.v1',
  ].sort(),
};

const COMMUNICATION_TRANSLATION_CONTRACT_CORE_INVENTORY = {
  ...COMMUNICATION_SUMMARY_BUILD_UNITS_INVENTORY,
  workflows: [
    ...COMMUNICATION_SUMMARY_BUILD_UNITS_INVENTORY.workflows,
    'communication_translation',
  ].sort(),
  businessCapabilities: [
    ...COMMUNICATION_SUMMARY_BUILD_UNITS_INVENTORY.businessCapabilities,
    'communication.translation.v1',
  ].sort(),
};

const COMMUNICATION_TRANSLATION_CROSS_OWNER_CONTRACTS_INVENTORY = {
  ...COMMUNICATION_TRANSLATION_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_TRANSLATION_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'ai.provider.translate.v1',
    'ai.translation.request.v1',
    'communication_translation.inference.v1',
    'communication_translation.source.blob.v1',
    'communication_translation.source_prepare.v1',
    'communication_translation.source_prepared.v1',
    'communication_translation.source_rejected.v1',
    'communications.ai-translation-source.blob.v1',
    'communications.ai-translation-source.v1',
  ].sort(),
};

const COMMUNICATION_TRANSLATION_PERSISTENCE_INVENTORY = {
  ...COMMUNICATION_TRANSLATION_CROSS_OWNER_CONTRACTS_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_TRANSLATION_CROSS_OWNER_CONTRACTS_INVENTORY.businessCapabilities,
    'communication_translation.storage.v1',
  ].sort(),
};

const COMMUNICATION_TRANSLATION_RUNTIME_INVENTORY = {
  ...COMMUNICATION_TRANSLATION_PERSISTENCE_INVENTORY,
};

const COMMUNICATION_EXPLANATION_CONTRACT_CORE_INVENTORY = {
  ...COMMUNICATION_TRANSLATION_RUNTIME_INVENTORY,
  workflows: [
    ...COMMUNICATION_TRANSLATION_RUNTIME_INVENTORY.workflows,
    'communication_explanation',
  ].sort(),
  businessCapabilities: [
    ...COMMUNICATION_TRANSLATION_RUNTIME_INVENTORY.businessCapabilities,
    'communication.explanation.v1',
  ].sort(),
};

const COMMUNICATION_EXPLANATION_CROSS_OWNER_CONTRACTS_INVENTORY = {
  ...COMMUNICATION_EXPLANATION_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_EXPLANATION_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'ai.explanation.request.v1',
    'ai.provider.explain.v1',
    'communication_explanation.inference.v1',
    'communication_explanation.source.blob.v1',
    'communication_explanation.source_prepare.v1',
    'communication_explanation.source_prepared.v1',
    'communication_explanation.source_rejected.v1',
    'communications.ai-explanation-source.blob.v1',
    'communications.ai-explanation-source.v1',
  ].sort(),
};

const COMMUNICATION_EXPLANATION_PERSISTENCE_INVENTORY = {
  ...COMMUNICATION_EXPLANATION_CROSS_OWNER_CONTRACTS_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_EXPLANATION_CROSS_OWNER_CONTRACTS_INVENTORY.businessCapabilities,
    'communication_explanation.storage.v1',
  ].sort(),
};

const COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY = {
  ...COMMUNICATION_EXPLANATION_PERSISTENCE_INVENTORY,
};

const COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_INVENTORY = {
  ...COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY,
  workflows: [
    ...COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY.workflows,
    'communication_recipient_suggestion',
  ].sort(),
  businessCapabilities: [
    ...COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY.businessCapabilities,
    'communication.recipient-suggestion.v1',
  ].sort(),
};

const COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_INVENTORY = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'communication_recipient_suggestion.source.blob.v1',
    'communication_recipient_suggestion.source_prepare.v1',
    'communication_recipient_suggestion.source_prepared.v1',
    'communication_recipient_suggestion.source_rejected.v1',
    'communications.recipient-source.v1',
  ].sort(),
};

const COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_INVENTORY = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_INVENTORY.businessCapabilities,
    'communication_recipient_suggestion.storage.v1',
  ].sort(),
};

const COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_INVENTORY = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_INVENTORY.businessCapabilities,
    'communications.recipient-source.blob.v1',
  ].sort(),
};

const COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_INVENTORY = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_INVENTORY,
  workflows: [
    ...COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_INVENTORY.workflows,
    'communication_task_candidate_extraction',
  ].sort(),
  businessCapabilities: [
    ...COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_INVENTORY.businessCapabilities,
    'communication.task-candidate-extraction.v1',
    'communication_task_candidate_extraction.source.blob.v1',
    'communication_task_candidate_extraction.source_prepare.v1',
    'communication_task_candidate_extraction.source_prepared.v1',
    'communication_task_candidate_extraction.source_rejected.v1',
    'communications.task-source.v1',
  ].sort(),
};

const COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_INVENTORY = {
  ...COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_INVENTORY.businessCapabilities,
    'communication_task_candidate_extraction.storage.v1',
  ].sort(),
};

const COMMUNICATION_TASK_CANDIDATE_SOURCE_PRODUCER_INVENTORY = {
  ...COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_INVENTORY.businessCapabilities,
    'communications.task-source.blob.v1',
  ].sort(),
};

const MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST = {
  'hermes-communication-cross-channel-forward-persistence': {
    default: [],
    'conformance-test-support': [],
  },
  'hermes-communication-delayed-delivery-persistence': {
    default: [],
    'conformance-test-support': [],
  },
  'hermes-communication-delivery-intent-persistence': {
    default: [],
    'conformance-test-support': [],
  },
  'hermes-mail-api': {
    default: [],
    'conformance-test-support': [],
  },
  'hermes-mail-imap': {
    default: [],
    'conformance-test-support': [],
  },
  'hermes-mail-gmail': {
    default: [],
    'conformance-test-support': ['hermes-mail-api/conformance-test-support'],
  },
  'hermes-mail-persistence': {
    default: [],
    'conformance-test-support': [],
  },
  'hermes-mail-runtime': {
    default: [],
    'conformance-test-support': [
      'hermes-mail-api/conformance-test-support',
      'hermes-mail-gmail/conformance-test-support',
      'hermes-mail-imap/conformance-test-support',
    ],
  },
};

const CLOCK_KEYS = ['wallTime', 'elapsedTime', 'testTime', 'moduleCapabilityEnabled'];

const EXIT_GATES = [
  'boots_without_external_services',
  'foundation_protocol_v1_conformance',
  'private_control_store_create_open_validate',
  'missing_or_invalid_store_recovery_only',
  'local_ipc_status_validate_export_shutdown',
  'pristine_inherited_fd_owner_enrollment',
  'server_bootstrap_pairing_tls_conformance',
  'file_release_authority_conformance',
  'managed_launch_toctou_conformance',
  'online_mutations_fail_closed',
  'exclusive_data_directory_lock',
  'bounded_shutdown',
  'wall_monotonic_fake_clock_conformance',
  'diagnostics_exclude_secrets_private_content',
];

const DEVELOPMENT_PROFILE_KEYS = [
  'id',
  'purpose',
  'workspaceRoot',
  'packages',
  'selection',
  'deviceProof',
  'privateKeyStorage',
  'persistentSecretsAllowed',
  'productDataAllowed',
  'networkListenerEnabled',
  'remotePairingEnabled',
  'externalServicesEnabled',
  'vaultEnabled',
  'releaseArtifactAllowed',
  'productionGateEvidenceAllowed',
  'visibleInsecureWarningRequired',
  'automaticProductionFallbackAllowed',
  'simulatedTargets',
];

const DEVELOPMENT_PACKAGE_KEYS = ['package', 'surface'];

function hasExactKeys(value, expectedKeys) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return false;
  const keys = Object.keys(value);
  return keys.length === expectedKeys.length
    && keys.every((key) => expectedKeys.includes(key));
}

function isExactOrderedStringList(value, expected) {
  return Array.isArray(expected)
    && Array.isArray(value)
    && value.length === expected.length
    && duplicates(value).length === 0
    && value.every((entry, index) => entry === expected[index]);
}

function isExactPackageInventory(packages, expectedPackages) {
  return Array.isArray(expectedPackages)
    && Array.isArray(packages)
    && packages.length === expectedPackages.length
    && packages.every((entry, index) => {
      const expected = expectedPackages[index];
      return hasExactKeys(entry, ['name', 'role', 'owner', 'surface'])
        && entry.name === expected.name
        && entry.role === expected.role
        && entry.owner === expected.owner
        && entry.surface === expected.surface;
    });
}

function isEmptyOwnerInventory(inventory) {
  const ownerClasses = [
    'domains',
    'integrations',
    'workflows',
    'engines',
    'businessCapabilities',
  ];
  return hasExactKeys(inventory, ownerClasses)
    && ownerClasses
      .every((ownerClass) => Array.isArray(inventory[ownerClass]) && inventory[ownerClass].length === 0);
}

function isExactOwnerInventory(inventory, expected) {
  const ownerClasses = [
    'domains',
    'integrations',
    'workflows',
    'engines',
    'businessCapabilities',
  ];
  return hasExactKeys(inventory, ownerClasses)
    && hasExactKeys(expected, ownerClasses)
    && ownerClasses.every((ownerClass) => (
      isExactOrderedStringList(inventory[ownerClass], expected[ownerClass])
    ));
}

function isExactWorkspaceDependencyAllowlist(allowlist, expectedPackages, expectedAllowlist) {
  if (!Array.isArray(expectedPackages) || !expectedAllowlist) return false;
  const packageNames = expectedPackages.map(({ name }) => name);
  return hasExactKeys(allowlist, packageNames)
    && packageNames.every((packageName) => isExactDependencyList(
      allowlist[packageName],
      expectedAllowlist[packageName],
    ));
}

function isExactDependencyList(actual, expected) {
  return Array.isArray(expected)
    && Array.isArray(actual)
    && actual.length === expected.length
    && actual.every((entry, index) => {
      const expectedEntry = expected[index];
      return hasExactKeys(entry, Object.keys(expectedEntry))
        && Object.entries(expectedEntry).every(([key, value]) => (
          Array.isArray(value)
            ? isExactOrderedStringList(entry[key], value)
            : entry[key] === value
        ));
    });
}

function isExactThirdPartyDependencyAllowlist(allowlist, expectedPackages, expectedAllowlist) {
  if (!Array.isArray(expectedPackages) || !expectedAllowlist) return false;
  const packageNames = expectedPackages.map(({ name }) => name);
  return hasExactKeys(allowlist, packageNames)
    && packageNames.every((packageName) => isExactDependencyList(
      allowlist[packageName],
      expectedAllowlist[packageName],
    ));
}

function isExactTargetPolicy(targetPolicy, expectedPackages) {
  if (!Array.isArray(expectedPackages)) return false;
  const packageNames = expectedPackages.map(({ name }) => name);
  if (!hasExactKeys(targetPolicy, packageNames)) return false;
  return packageNames.every((packageName) => {
    const target = targetPolicy[packageName];
    const packageDescriptor = expectedPackages.find(({ name }) => name === packageName);
    const protocolPackage = [
      'hermes-events-protocol',
      'hermes-runtime-protocol',
      'hermes-gateway-protocol',
      'hermes-storage-protocol',
      'hermes-scheduler-protocol',
      'hermes-whatsapp-api',
      'hermes-telegram-api',
      'hermes-zulip-api',
      'hermes-mail-api',
      'hermes-communications-ingress',
      'hermes-communications-call-evidence-ingress',
      'hermes-communications-call-evidence-api',
      'hermes-communications-attachment-contract',
      'hermes-communications-api',
      'hermes-communications-content-api',
      'hermes-communications-saved-query-api',
      'hermes-communications-sender-insights-api',
      'hermes-communications-evidence-export-source-api',
      'hermes-communications-cross-channel-forward-source-api',
      'hermes-communications-ai-source-api',
      'hermes-communication-reply-suggestion-api',
      'hermes-communication-summary-api',
      'hermes-communication-translation-api',
      'hermes-communication-explanation-api',
      'hermes-communication-recipient-suggestion-api',
      'hermes-communications-recipient-source-api',
      'hermes-communication-task-candidate-api',
      'hermes-communications-task-source-api',
      'hermes-ai-contracts',
      'hermes-attachment-archive-inspection-api',
      'hermes-attachment-archive-inspection-ingress',
      'hermes-communications-export-api',
      'hermes-communication-delivery-intent-api',
      'hermes-communication-delivery-intent-ingress-api',
      'hermes-communication-bulk-action-api',
      'hermes-communication-delayed-delivery-api',
      'hermes-communication-cross-channel-forward-api',
      'hermes-review-attention-api',
      'hermes-mail-delivery-intent-contract',
      'hermes-telegram-delivery-intent-contract',
      'hermes-whatsapp-delivery-intent-contract',
      'hermes-zulip-delivery-intent-contract',
      'hermes-attachment-security-contract',
    ].includes(packageName);
    return hasExactKeys(target, ['primaryKind', 'customBuildAllowed'])
      && target.primaryKind === (
        ['runtime', 'assembly'].includes(packageDescriptor?.surface) ? 'bin' : 'lib'
      )
      && target.customBuildAllowed === protocolPackage;
  });
}

function isExactCargoFeatureAllowlist(actual, expected) {
  if (!hasExactKeys(actual, Object.keys(expected))) return false;
  return Object.entries(expected).every(([packageName, expectedFeatures]) => {
    const actualFeatures = actual[packageName];
    if (!hasExactKeys(actualFeatures, Object.keys(expectedFeatures))) return false;
    return Object.entries(expectedFeatures).every(([featureName, featureMembers]) => (
      isExactOrderedStringList(actualFeatures[featureName], featureMembers)
    ));
  });
}

function expectedSlice(currentSlice) {
  if (currentSlice === 'kernel_recovery_only_v1') {
    return {
      profile: KERNEL_PROFILE,
      packages: RECOVERY_PRODUCTION_PACKAGES,
      workspaceDependencies: RECOVERY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: RECOVERY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: RECOVERY_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'module_control_plane_v1') {
    return {
      profile: MODULE_CONTROL_PROFILE,
      packages: RECOVERY_PRODUCTION_PACKAGES,
      workspaceDependencies: RECOVERY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: RECOVERY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: RECOVERY_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'server_bootstrap_pairing_v1') {
    return {
      profile: SERVER_BOOTSTRAP_PAIRING_PROFILE,
      packages: RECOVERY_PRODUCTION_PACKAGES,
      workspaceDependencies: RECOVERY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: RECOVERY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: RECOVERY_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'managed_launch_trust_v1') {
    return {
      profile: MANAGED_LAUNCH_TRUST_PROFILE,
      packages: RECOVERY_PRODUCTION_PACKAGES,
      workspaceDependencies: RECOVERY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: RECOVERY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: RECOVERY_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'vault_foundation_v1' || currentSlice === 'vault_v1') {
    return {
      profile: MANAGED_LAUNCH_TRUST_PROFILE,
      packages: VAULT_FOUNDATION_PRODUCTION_PACKAGES,
      workspaceDependencies: VAULT_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: VAULT_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: VAULT_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'clock_v1') {
    return {
      profile: MANAGED_LAUNCH_TRUST_PROFILE,
      packages: CLOCK_PRODUCTION_PACKAGES,
      workspaceDependencies: CLOCK_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: CLOCK_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: VAULT_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'telemetry_foundation_v1') {
    return {
      profile: MANAGED_LAUNCH_TRUST_PROFILE,
      packages: TELEMETRY_FOUNDATION_PRODUCTION_PACKAGES,
      workspaceDependencies: TELEMETRY_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: TELEMETRY_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: VAULT_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'storage_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: STORAGE_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: STORAGE_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: STORAGE_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'nats_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: NATS_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: NATS_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: NATS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'blob_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: BLOB_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: BLOB_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: BLOB_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'blob_runtime_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: BLOB_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: BLOB_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: BLOB_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'scheduler_protocol_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: SCHEDULER_PROTOCOL_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: SCHEDULER_PROTOCOL_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: SCHEDULER_PROTOCOL_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'scheduler_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: SCHEDULER_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: SCHEDULER_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: SCHEDULER_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'scheduler_persistence_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: SCHEDULER_PERSISTENCE_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: SCHEDULER_PERSISTENCE_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: SCHEDULER_PERSISTENCE_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'gateway_session_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: GATEWAY_SESSION_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: GATEWAY_SESSION_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: GATEWAY_SESSION_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'scheduler_receipt_delivery_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'scheduler_jetstream_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: SCHEDULER_JETSTREAM_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: SCHEDULER_JETSTREAM_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: SCHEDULER_JETSTREAM_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'scheduler_runtime_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: SCHEDULER_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: SCHEDULER_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: SCHEDULER_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'gateway_runtime_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: GATEWAY_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: GATEWAY_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: GATEWAY_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'gateway_runtime_plus_mail_telegram_whatsapp_communications_v1') {
    return {
      profile: MANAGED_LAUNCH_TRUST_PROFILE,
      packages: MAIL_COMMUNICATIONS_FOUNDATION_PRODUCTION_PACKAGES,
      workspaceDependencies: MAIL_COMMUNICATIONS_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: MAIL_COMMUNICATIONS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'first_owner_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: FIRST_OWNER_INVENTORY,
      packages: FIRST_OWNER_PRODUCTION_PACKAGES,
      workspaceDependencies: FIRST_OWNER_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: FIRST_OWNER_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_security_engine_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_SECURITY_ENGINE_INVENTORY,
      packages: ATTACHMENT_SECURITY_ENGINE_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_SECURITY_ENGINE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_SECURITY_ENGINE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'mail_outbound_mime_attachments_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: MAIL_OUTBOUND_MIME_ATTACHMENTS_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: MAIL_OUTBOUND_MIME_ATTACHMENTS_PRODUCTION_PACKAGES,
      workspaceDependencies: MAIL_OUTBOUND_MIME_ATTACHMENTS_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: MAIL_OUTBOUND_MIME_ATTACHMENTS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communications_content_read_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_CONTENT_READ_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_CONTENT_READ_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATIONS_CONTENT_READ_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATIONS_CONTENT_READ_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communications_saved_search_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_SAVED_SEARCH_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_SAVED_SEARCH_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATIONS_SAVED_SEARCH_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATIONS_SAVED_SEARCH_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communications_sender_insights_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_SENDER_INSIGHTS_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_SENDER_INSIGHTS_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATIONS_SENDER_INSIGHTS_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATIONS_SENDER_INSIGHTS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communications_export_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_EXPORT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_EXPORT_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATIONS_EXPORT_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATIONS_EXPORT_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delivery_intent_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_EXPORT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delivery_intent_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_EXPORT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delivery_intent_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_EXPORT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELIVERY_INTENT_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELIVERY_INTENT_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELIVERY_INTENT_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delivery_intent_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_EXPORT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'provider_delivery_intent_contracts_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_EXPORT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'delivery_intent_transactional_event_adapters_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_EXPORT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_PRODUCTION_PACKAGES,
      workspaceDependencies:
        DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'delivery_intent_target_bound_blob_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_PRODUCTION_PACKAGES,
      workspaceDependencies:
        DELIVERY_INTENT_TARGET_BOUND_BLOB_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        DELIVERY_INTENT_TARGET_BOUND_BLOB_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_bulk_action_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_BULK_ACTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_BULK_ACTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_BULK_ACTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_bulk_action_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_BULK_ACTION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_BULK_ACTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_BULK_ACTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_bulk_action_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_BULK_ACTION_RUNTIME_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_BULK_ACTION_RUNTIME_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_BULK_ACTION_RUNTIME_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (
    currentSlice === 'communication_bulk_action_assembly_v1'
    || currentSlice === 'communication_bulk_action_v1'
  ) {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_BULK_ACTION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_BULK_ACTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_BULK_ACTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delayed_delivery_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delayed_delivery_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (
    currentSlice === 'communication_delayed_delivery_runtime_adapters_v1'
    || currentSlice === 'communication_delayed_delivery_due_event_adapter_v1'
    || currentSlice === 'communication_delayed_delivery_store_adapter_v1'
    || currentSlice === 'communication_delayed_delivery_persistence_runtime_surfaces_v1'
  ) {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: [
        'communication_delayed_delivery_store_adapter_v1',
        'communication_delayed_delivery_persistence_runtime_surfaces_v1',
      ].includes(currentSlice)
        ? COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_PRODUCTION_PACKAGES
        : COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_PRODUCTION_PACKAGES,
      workspaceDependencies:
        [
          'communication_delayed_delivery_store_adapter_v1',
          'communication_delayed_delivery_persistence_runtime_surfaces_v1',
        ].includes(currentSlice)
          ? COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST
          : COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        [
          'communication_delayed_delivery_store_adapter_v1',
          'communication_delayed_delivery_persistence_runtime_surfaces_v1',
        ].includes(currentSlice)
          ? COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST
          : COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delayed_delivery_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delayed_delivery_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_cross_channel_forward_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_cross_channel_forward_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_cross_channel_forward_source_contract_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delivery_intent_ingress_contract_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_cross_channel_forward_event_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_cross_channel_forward_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_cross_channel_forward_terminal_results_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_CONSUMER_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_cross_channel_forward_client_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communications_call_evidence_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communications_call_evidence_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communications_call_evidence_managed_consumer_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATIONS_CALL_EVIDENCE_MANAGED_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communications_call_evidence_query_realtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_communications_attention_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (
    currentSlice === 'review_communications_attention_persistence_v1'
    || currentSlice === 'review_communications_attention_read_realtime_persistence_v1'
  ) {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_communications_attention_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies:
        REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_communications_attention_release_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies:
        REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_communications_attention_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_AI_SOURCE_CONTRACT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_AI_SOURCE_CONTRACT_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATIONS_AI_SOURCE_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATIONS_AI_SOURCE_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_archive_inspection_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (
    currentSlice === 'attachment_archive_inspection_persistence_join_v1'
    || currentSlice === 'blob_current_custodian_redelegation_v1'
    || currentSlice === 'attachment_archive_inspection_ingress_contract_v1'
    || currentSlice === 'attachment_archive_inspection_event_replay_persistence_v1'
    || currentSlice === 'attachment_security_archive_delegation_runtime_v1'
  ) {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_archive_inspection_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_archive_inspection_release_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (
    currentSlice === 'attachment_archive_inspection_v1'
    || currentSlice === 'ollama_ai_provider_v1'
    || currentSlice === 'ai_inference_v1'
    || currentSlice === 'communication_reply_suggestion_v1'
  ) {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_ARCHIVE_INSPECTION_CLIENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_summary_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_SUMMARY_BUILD_UNITS_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_SUMMARY_BUILD_UNITS_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_SUMMARY_BUILD_UNITS_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_SUMMARY_BUILD_UNITS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_translation_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TRANSLATION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TRANSLATION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TRANSLATION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TRANSLATION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_translation_cross_owner_contracts_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TRANSLATION_CROSS_OWNER_CONTRACTS_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TRANSLATION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TRANSLATION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TRANSLATION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_translation_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TRANSLATION_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TRANSLATION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TRANSLATION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TRANSLATION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_translation_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TRANSLATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TRANSLATION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TRANSLATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TRANSLATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_translation_ai_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TRANSLATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TRANSLATION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TRANSLATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TRANSLATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_translation_ollama_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TRANSLATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TRANSLATION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TRANSLATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TRANSLATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_translation_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TRANSLATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TRANSLATION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TRANSLATION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TRANSLATION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_EXPLANATION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_EXPLANATION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_EXPLANATION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_EXPLANATION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_cross_owner_contracts_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_EXPLANATION_CROSS_OWNER_CONTRACTS_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_EXPLANATION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_EXPLANATION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_EXPLANATION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_EXPLANATION_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_EXPLANATION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_EXPLANATION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_EXPLANATION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_EXPLANATION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_EXPLANATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_EXPLANATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_ai_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_EXPLANATION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_EXPLANATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_EXPLANATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_ollama_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_EXPLANATION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_EXPLANATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_EXPLANATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_EXPLANATION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_EXPLANATION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_EXPLANATION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_managed_conformance_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_EXPLANATION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_EXPLANATION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_EXPLANATION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_recipient_suggestion_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_recipient_suggestion_source_contract_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_recipient_suggestion_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_recipient_suggestion_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_recipient_suggestion_source_producer_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_recipient_suggestion_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_recipient_suggestion_managed_conformance_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_live_provider_conformance_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_task_candidate_contract_core_source_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_task_candidate_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_task_candidate_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TASK_CANDIDATE_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TASK_CANDIDATE_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TASK_CANDIDATE_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_task_candidate_source_producer_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TASK_CANDIDATE_SOURCE_PRODUCER_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TASK_CANDIDATE_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TASK_CANDIDATE_SOURCE_PRODUCER_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TASK_CANDIDATE_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  return null;
}

function isExactDevelopmentProfile(profile) {
  return hasExactKeys(profile, DEVELOPMENT_PROFILE_KEYS)
    && profile.id === 'development_full_platform_v1'
    && profile.purpose === 'full_local_platform_development_with_simulated_trust'
    && profile.workspaceRoot === 'development'
    && Array.isArray(profile.packages)
    && profile.packages.length === 2
    && profile.packages.every((entry) => hasExactKeys(entry, DEVELOPMENT_PACKAGE_KEYS))
    && profile.packages[0].package === 'hermes-development-kernel-operator'
    && profile.packages[0].surface === 'runtime'
    && profile.packages[1].package === 'hermes-development-assembly'
    && profile.packages[1].surface === 'assembly'
    && profile.selection === 'explicit_development_invocation_only'
    && profile.deviceProof === 'file_adapter_es256'
    && profile.privateKeyStorage === 'owner_private_file_adapter'
    && profile.persistentSecretsAllowed === true
    && profile.productDataAllowed === true
    && profile.networkListenerEnabled === true
    && profile.remotePairingEnabled === true
    && profile.externalServicesEnabled === true
    && profile.vaultEnabled === true
    && profile.releaseArtifactAllowed === false
    && profile.productionGateEvidenceAllowed === false
    && profile.visibleInsecureWarningRequired === true
    && profile.automaticProductionFallbackAllowed === false
    && isExactOrderedStringList(profile.simulatedTargets, [
      'macos_tauri_embedded_v1',
      'linux_docker_server_v1',
    ]);
}

function isExactClock(clock) {
  return hasExactKeys(clock, CLOCK_KEYS)
    && clock.wallTime === 'system_time_utc_timestamps_only'
    && clock.elapsedTime === 'monotonic_deadlines_and_timeouts'
    && clock.testTime === 'injected_deterministic_fake'
    && clock.moduleCapabilityEnabled === false;
}

function isExactKernelProfile(profile, constitutionalComponents, expected) {
  return expected !== null
    && expected !== undefined
    && hasExactKeys(profile, KERNEL_PROFILE_KEYS)
    && profile.maximumState === expected.maximumState
    && isExactOrderedStringList(profile.allowedStates, expected.allowedStates)
    && isExactOrderedStringList(profile.forbiddenStates, expected.forbiddenStates)
    && isExactOrderedStringList(profile.activeComponents, expected.activeComponents)
    && profile.activeComponents.every((component) => constitutionalComponents.includes(component))
    && profile.transport === expected.transport
    && isExactOrderedStringList(profile.onlineOperations, expected.onlineOperations)
    && isExactOrderedStringList(profile.bootstrapOperations, expected.bootstrapOperations)
    && isExactOrderedStringList(profile.offlineOperations, expected.offlineOperations)
    && isExactOrderedStringList(profile.externalServices, expected.externalServices)
    && isExactOrderedStringList(profile.managedChildren, expected.managedChildren)
    && profile.publicGatewayEnabled === (expected.publicGatewayEnabled ?? false)
    && profile.networkListenerEnabled === expected.networkListenerEnabled
    && profile.moduleRegistrationEnabled === expected.moduleRegistrationEnabled
    && profile.managedLaunchEnabled === expected.managedLaunchEnabled
    && profile.natsDataPlaneEnabled === (expected.natsDataPlaneEnabled ?? false)
    && profile.businessDataPlaneEnabled === (expected.businessDataPlaneEnabled ?? false)
    && profile.wholeInstanceBackupEnabled === (expected.wholeInstanceBackupEnabled ?? false)
    && isExactClock(profile.clock);
}

export function validateImplementationSlicePolicy(policy) {
  const implementation = policy?.implementation;
  const slice = expectedSlice(implementation?.currentSlice);
  const checks = {
    implementation_keys: hasExactKeys(implementation, IMPLEMENTATION_KEYS),
    supported_slice: slice !== null,
    package_mode: implementation?.productionPackageMode === 'exact_allowlist',
    package_inventory: isExactPackageInventory(implementation?.productionPackages, slice?.packages),
    workspace_dependencies: isExactWorkspaceDependencyAllowlist(
      implementation?.workspaceDependencyAllowlist,
      slice?.packages,
      slice?.workspaceDependencies,
    ),
    third_party_dependencies: isExactThirdPartyDependencyAllowlist(
      implementation?.thirdPartyDependencyAllowlist,
      slice?.packages,
      slice?.thirdPartyDependencies,
    ),
    forbidden_dependencies: isExactOrderedStringList(
      implementation?.forbiddenDependencies,
      FORBIDDEN_DEPENDENCIES,
    ),
    forbidden_dependency_prefixes: isExactOrderedStringList(
      implementation?.forbiddenDependencyPrefixes,
      slice?.forbiddenDependencyPrefixes,
    ),
    cargo_features: implementation?.cargoFeaturesEnabled === false,
    cargo_feature_allowlist: isExactCargoFeatureAllowlist(
      implementation?.cargoFeatureAllowlist,
      slice?.cargoFeatures ?? {},
    ),
    target_policy: isExactTargetPolicy(implementation?.targetPolicy, slice?.packages),
    development_profile: isExactDevelopmentProfile(implementation?.developmentProfile),
    owner_inventory: slice?.ownerInventory
      ? isExactOwnerInventory(implementation?.ownerInventory, slice.ownerInventory)
      : isEmptyOwnerInventory(implementation?.ownerInventory),
    kernel_profile: isExactKernelProfile(
      implementation?.kernelProfile,
      list(policy?.kernel?.constitutionalComponents),
      slice?.profile,
    ),
    exit_gates: isExactOrderedStringList(implementation?.exitGates, EXIT_GATES),
  };
  const invalidChecks = Object.entries(checks)
    .filter(([, valid]) => !valid)
    .map(([name]) => name);

  return invalidChecks.length === 0 ? [] : [violation(
    'implementation_slice_policy',
    'implementation',
    `current implementation must remain the exact authorized Kernel slice; invalid=${invalidChecks.join(',')}`,
  )];
}
