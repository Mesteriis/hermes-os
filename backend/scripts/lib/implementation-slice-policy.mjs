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
  { name: 'hermes-telegram-api', role: 'integration', owner: 'telegram', surface: 'contract' },
  { name: 'hermes-telegram-core', role: 'integration', owner: 'telegram', surface: 'implementation' },
  { name: 'hermes-telegram-tdlib', role: 'integration', owner: 'telegram', surface: 'implementation' },
  { name: 'hermes-telegram-persistence', role: 'integration', owner: 'telegram', surface: 'persistence' },
  { name: 'hermes-telegram-runtime', role: 'integration', owner: 'telegram', surface: 'runtime' },
  { name: 'hermes-whatsapp-api', role: 'integration', owner: 'whatsapp', surface: 'contract' },
  { name: 'hermes-whatsapp-core', role: 'integration', owner: 'whatsapp', surface: 'implementation' },
  { name: 'hermes-whatsapp-persistence', role: 'integration', owner: 'whatsapp', surface: 'persistence' },
  { name: 'hermes-whatsapp-runtime', role: 'integration', owner: 'whatsapp', surface: 'runtime' },
  { name: 'hermes-zulip-api', role: 'integration', owner: 'zulip', surface: 'contract' },
  { name: 'hermes-zulip-core', role: 'integration', owner: 'zulip', surface: 'implementation' },
  { name: 'hermes-zulip-http', role: 'integration', owner: 'zulip', surface: 'implementation' },
  { name: 'hermes-zulip-persistence', role: 'integration', owner: 'zulip', surface: 'persistence' },
  { name: 'hermes-zulip-runtime', role: 'integration', owner: 'zulip', surface: 'runtime' },
  { name: 'hermes-communications-ingress', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'hermes-communications-api', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'hermes-communications-domain', role: 'domain', owner: 'communications', surface: 'implementation' },
  { name: 'hermes-communications-persistence', role: 'domain', owner: 'communications', surface: 'persistence' },
  { name: 'hermes-communications-runtime', role: 'domain', owner: 'communications', surface: 'runtime' },
];

const FIRST_OWNER_PRODUCTION_PACKAGES = [
  ...GATEWAY_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'hermes-communications-ingress', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'hermes-communications-api', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'hermes-communications-domain', role: 'domain', owner: 'communications', surface: 'implementation' },
  { name: 'hermes-communications-persistence', role: 'domain', owner: 'communications', surface: 'persistence' },
  { name: 'hermes-communications-runtime', role: 'domain', owner: 'communications', surface: 'runtime' },
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
    { name: 'hermes-runtime-protocol', kind: 'normal' },
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
  'hermes-mail-gmail': [],
  'hermes-mail-smtp': [
    { name: 'hermes-mail-api', kind: 'normal' },
  ],
  'hermes-mail-persistence': [
    { name: 'hermes-events-protocol', kind: 'normal' },
    { name: 'hermes-storage-protocol', kind: 'normal' },
  ],
  'hermes-mail-runtime': [
    { name: 'hermes-mail-api', kind: 'normal' },
    { name: 'hermes-mail-core', kind: 'normal' },
    { name: 'hermes-mail-imap', kind: 'normal' },
    { name: 'hermes-mail-gmail', kind: 'normal' },
    { name: 'hermes-mail-smtp', kind: 'normal' },
    { name: 'hermes-mail-persistence', kind: 'normal' },
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

const FIRST_OWNER_WORKSPACE_DEPENDENCY_ALLOWLIST = Object.fromEntries(
  FIRST_OWNER_PRODUCTION_PACKAGES.map(({ name }) => [
    name,
    MAIL_COMMUNICATIONS_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST[name],
  ]),
);

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
  'hermes-runtime-protocol': PROTOCOL_THIRD_PARTY_DEPENDENCIES,
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
  ],
  'hermes-mail-imap': [
    { name: 'async-imap', kind: 'normal', source: 'crates_io', version: '=0.11.2', defaultFeatures: true, features: [] },
    { name: 'async-native-tls', kind: 'normal', source: 'crates_io', version: '=0.6.0', defaultFeatures: true, features: [] },
    { name: 'async-std', kind: 'normal', source: 'crates_io', version: '=1.13.2', defaultFeatures: true, features: [] },
    { name: 'futures-util', kind: 'normal', source: 'crates_io', version: '=0.3.32', defaultFeatures: true, features: [] },
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
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'hermes-mail-runtime': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
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
  'hermes-whatsapp-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['alloc', 'derive'] },
  ],
  'hermes-whatsapp-core': [],
  'hermes-whatsapp-persistence': [
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'hermes-whatsapp-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread'] },
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
};

const FIRST_OWNER_THIRD_PARTY_DEPENDENCY_ALLOWLIST = Object.fromEntries(
  FIRST_OWNER_PRODUCTION_PACKAGES.map(({ name }) => [
    name,
    MAIL_COMMUNICATIONS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST[name],
  ]),
);

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
    'communications.blob.v1',
    'communications.events.v1',
    'communications.observe.v1',
    'communications.query.v1',
    'communications.search.index.v1',
    'communications.storage.v1',
  ],
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
  'package',
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
      'hermes-communications-api',
    ].includes(packageName);
    return hasExactKeys(target, ['primaryKind', 'customBuildAllowed'])
      && target.primaryKind === (packageDescriptor?.surface === 'runtime' ? 'bin' : 'lib')
      && target.customBuildAllowed === protocolPackage;
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
  return null;
}

function isExactDevelopmentProfile(profile) {
  return hasExactKeys(profile, DEVELOPMENT_PROFILE_KEYS)
    && profile.id === 'development_full_platform_v1'
    && profile.purpose === 'full_local_platform_development_with_simulated_trust'
    && profile.workspaceRoot === 'development/runtime'
    && profile.package === 'hermes-development-kernel-operator'
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
