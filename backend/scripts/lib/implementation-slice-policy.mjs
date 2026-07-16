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

const PRODUCTION_PACKAGES = [
  { name: 'hermes-events-protocol', role: 'platform', owner: 'events', surface: 'contract' },
  { name: 'hermes-runtime-protocol', role: 'platform', owner: 'runtime_protocol', surface: 'contract' },
  { name: 'hermes-gateway-protocol', role: 'api', owner: 'gateway', surface: 'contract' },
  { name: 'hermes-kernel-control-store', role: 'core', owner: 'kernel', surface: 'contract' },
  { name: 'hermes-kernel-control-store-sqlite', role: 'core', owner: 'kernel', surface: 'persistence' },
  { name: 'hermes-kernel', role: 'core', owner: 'kernel', surface: 'runtime' },
];

const WORKSPACE_DEPENDENCY_ALLOWLIST = {
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

const THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  'hermes-events-protocol': PROTOCOL_THIRD_PARTY_DEPENDENCIES,
  'hermes-runtime-protocol': PROTOCOL_THIRD_PARTY_DEPENDENCIES,
  'hermes-gateway-protocol': PROTOCOL_THIRD_PARTY_DEPENDENCIES,
  'hermes-kernel-control-store': [],
  'hermes-kernel-control-store-sqlite': [
    {
      name: 'rusqlite', kind: 'normal', source: 'crates_io', version: '=0.40.1', defaultFeatures: false, features: ['backup', 'bundled'],
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
      name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [],
    },
    {
      name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [],
    },
    {
      name: 'signal-hook', kind: 'normal', source: 'crates_io', version: '=0.3.18', defaultFeatures: true, features: [],
    },
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

const FORBIDDEN_DEPENDENCY_PREFIXES = [
  'hermes-vault-',
  'hermes-storage-',
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
};

const CLOCK_KEYS = ['wallTime', 'elapsedTime', 'testTime', 'moduleCapabilityEnabled'];

const EXIT_GATES = [
  'boots_without_external_services',
  'foundation_protocol_v1_conformance',
  'private_control_store_create_open_validate',
  'missing_or_invalid_store_recovery_only',
  'local_ipc_status_validate_export_shutdown',
  'pristine_inherited_fd_owner_enrollment',
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
  return Array.isArray(value)
    && value.length === expected.length
    && duplicates(value).length === 0
    && value.every((entry, index) => entry === expected[index]);
}

function isExactPackageInventory(packages) {
  return Array.isArray(packages)
    && packages.length === PRODUCTION_PACKAGES.length
    && packages.every((entry, index) => {
      const expected = PRODUCTION_PACKAGES[index];
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

function isExactWorkspaceDependencyAllowlist(allowlist) {
  const packageNames = PRODUCTION_PACKAGES.map(({ name }) => name);
  return hasExactKeys(allowlist, packageNames)
    && packageNames.every((packageName) => isExactDependencyList(
      allowlist[packageName],
      WORKSPACE_DEPENDENCY_ALLOWLIST[packageName],
    ));
}

function isExactDependencyList(actual, expected) {
  return Array.isArray(actual)
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

function isExactThirdPartyDependencyAllowlist(allowlist) {
  const packageNames = PRODUCTION_PACKAGES.map(({ name }) => name);
  return hasExactKeys(allowlist, packageNames)
    && packageNames.every((packageName) => isExactDependencyList(
      allowlist[packageName],
      THIRD_PARTY_DEPENDENCY_ALLOWLIST[packageName],
    ));
}

function isExactTargetPolicy(targetPolicy) {
  const packageNames = PRODUCTION_PACKAGES.map(({ name }) => name);
  if (!hasExactKeys(targetPolicy, packageNames)) return false;
  return packageNames.every((packageName) => {
    const target = targetPolicy[packageName];
    const protocolPackage = [
      'hermes-events-protocol',
      'hermes-runtime-protocol',
      'hermes-gateway-protocol',
    ].includes(packageName);
    return hasExactKeys(target, ['primaryKind', 'customBuildAllowed'])
      && target.primaryKind === (packageName === 'hermes-kernel' ? 'bin' : 'lib')
      && target.customBuildAllowed === protocolPackage;
  });
}

function isExactDevelopmentProfile(profile) {
  return hasExactKeys(profile, DEVELOPMENT_PROFILE_KEYS)
    && profile.id === 'development_full_platform_v1'
    && profile.purpose === 'full_local_platform_development_with_simulated_trust'
    && profile.workspaceRoot === 'development/runtime'
    && profile.package === 'hermes-development-platform-runtime'
    && profile.selection === 'explicit_development_invocation_only'
    && profile.deviceProof === 'software_es256_development_only'
    && profile.privateKeyStorage === 'development_local_filesystem'
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

function isExactKernelProfile(profile, constitutionalComponents) {
  return hasExactKeys(profile, KERNEL_PROFILE_KEYS)
    && profile.maximumState === KERNEL_PROFILE.maximumState
    && isExactOrderedStringList(profile.allowedStates, KERNEL_PROFILE.allowedStates)
    && isExactOrderedStringList(profile.forbiddenStates, KERNEL_PROFILE.forbiddenStates)
    && isExactOrderedStringList(profile.activeComponents, KERNEL_PROFILE.activeComponents)
    && profile.activeComponents.every((component) => constitutionalComponents.includes(component))
    && profile.transport === KERNEL_PROFILE.transport
    && isExactOrderedStringList(profile.onlineOperations, KERNEL_PROFILE.onlineOperations)
    && isExactOrderedStringList(profile.bootstrapOperations, KERNEL_PROFILE.bootstrapOperations)
    && isExactOrderedStringList(profile.offlineOperations, KERNEL_PROFILE.offlineOperations)
    && isExactOrderedStringList(profile.externalServices, KERNEL_PROFILE.externalServices)
    && isExactOrderedStringList(profile.managedChildren, KERNEL_PROFILE.managedChildren)
    && profile.publicGatewayEnabled === false
    && profile.networkListenerEnabled === false
    && profile.moduleRegistrationEnabled === false
    && profile.managedLaunchEnabled === false
    && profile.natsDataPlaneEnabled === false
    && profile.businessDataPlaneEnabled === false
    && profile.wholeInstanceBackupEnabled === false
    && isExactClock(profile.clock);
}

export function validateImplementationSlicePolicy(policy) {
  const implementation = policy?.implementation;
  const valid = hasExactKeys(implementation, IMPLEMENTATION_KEYS)
    && implementation.currentSlice === 'kernel_recovery_only_v1'
    && implementation.productionPackageMode === 'exact_allowlist'
    && isExactPackageInventory(implementation.productionPackages)
    && isExactWorkspaceDependencyAllowlist(implementation.workspaceDependencyAllowlist)
    && isExactThirdPartyDependencyAllowlist(
      implementation.thirdPartyDependencyAllowlist,
    )
    && isExactOrderedStringList(
      implementation.forbiddenDependencies,
      FORBIDDEN_DEPENDENCIES,
    )
    && isExactOrderedStringList(
      implementation.forbiddenDependencyPrefixes,
      FORBIDDEN_DEPENDENCY_PREFIXES,
    )
    && implementation.cargoFeaturesEnabled === false
    && isExactTargetPolicy(implementation.targetPolicy)
    && isExactDevelopmentProfile(implementation.developmentProfile)
    && isEmptyOwnerInventory(implementation.ownerInventory)
    && isExactKernelProfile(
      implementation.kernelProfile,
      list(policy?.kernel?.constitutionalComponents),
    )
    && isExactOrderedStringList(implementation.exitGates, EXIT_GATES);

  return valid ? [] : [violation(
    'implementation_slice_policy',
    'implementation',
    'current implementation must remain the exact recovery-only Kernel slice authorized by ADR-0225',
  )];
}
