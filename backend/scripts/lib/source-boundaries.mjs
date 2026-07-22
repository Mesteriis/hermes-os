import {
  ownerAliases,
  pathTokens,
  violation,
} from './validation-diagnostics.mjs';
import { sqlReferencedIdentifiers } from './sql-inspection.mjs';

function forbiddenOwnerTokens(policy) {
  return new Set([
    ...policy.domains.blocked,
    ...policy.projections.blockedOwners,
  ]);
}

function sqlOwnerTokens(policy) {
  const values = forbiddenOwnerTokens(policy);
  const expanded = new Set();
  for (const value of values) for (const alias of ownerAliases(value)) expanded.add(alias);
  return expanded;
}

function declaredOwnerTokens(path, markers) {
  const segments = path.toLowerCase().split(/[\\/]+/u).filter(Boolean);
  const tokens = new Set();
  for (let index = 0; index < segments.length - 1; index += 1) {
    if (!markers.has(segments[index])) continue;
    for (const token of pathTokens(segments[index + 1])) tokens.add(token);
  }
  return tokens;
}

function globMatches(pattern, value) {
  const escaped = pattern.toLowerCase().replace(/[.+?^${}()|[\]\\]/gu, '\\$&');
  const expression = escaped.replaceAll('*', '.*');
  return new RegExp(`^${expression}$`, 'u').test(value.toLowerCase());
}

export function validateSourceEntries(policy, entries) {
  const violations = [];
  const blockedDomains = new Set(policy.domains.blocked);
  const blockedProjections = new Set(policy.projections.blockedOwners);
  const blockedSql = sqlOwnerTokens(policy);
  const sqlExtensions = new Set(policy.source.sqlExtensions);
  const ownerPathMarkers = new Set(policy.source.ownerPathMarkers);
  const forbiddenTestDirectories = new Set(policy.tests?.forbiddenProductionDirectories ?? []);
  const forbiddenTestFilePatterns = policy.tests?.forbiddenProductionFilePatterns ?? [];
  const emitted = new Set();

  function emit(code, location, message) {
    const key = `${code}\0${location}\0${message}`;
    if (!emitted.has(key)) violations.push(violation(code, location, message));
    emitted.add(key);
  }

  for (const entry of entries) {
    if (policy.source.forbidSymlinks === true && entry.isSymbolicLink === true) {
      emit(
        'source_symlink',
        entry.path,
        'production source symlinks are forbidden by ADR-0211',
      );
    }

    const pathSegments = entry.path.split(/[\\/]+/u).filter(Boolean);
    const fileName = pathSegments.at(-1) ?? '';
    const containsTestDirectory = pathSegments.some(
      (segment) => forbiddenTestDirectories.has(segment.toLowerCase()),
    );
    const matchesTestFile = forbiddenTestFilePatterns.some(
      (pattern) => globMatches(pattern, fileName),
    );
    if (containsTestDirectory || matchesTestFile) {
      emit(
        'test_in_production_source',
        entry.path,
        'backend test code must live under the dedicated backend/tests root',
      );
    }

    const tokens = declaredOwnerTokens(entry.path, ownerPathMarkers);
    for (const owner of blockedDomains) {
      if ([...ownerAliases(owner)].some((alias) => tokens.has(alias))) {
        emit('blocked_source_owner', entry.path, `source path declares blocked domain ${owner}`);
      }
    }
    for (const owner of blockedProjections) {
      if ([...ownerAliases(owner)].some((alias) => tokens.has(alias))) {
        emit('blocked_projection', entry.path, `source path declares blocked projection ${owner}`);
      }
    }

    const extension = entry.path.includes('.') ? `.${entry.path.split('.').pop().toLowerCase()}` : '';
    if (!sqlExtensions.has(extension) || typeof entry.content !== 'string') continue;

    for (const identifier of sqlReferencedIdentifiers(entry.content)) {
      const identifierTokens = pathTokens(identifier);
      const blocked = identifierTokens.find((token) => blockedSql.has(token));
      if (blocked) emit('blocked_sql_owner', entry.path, `SQL owns blocked identifier ${identifier}`);
    }
  }

  return violations;
}
