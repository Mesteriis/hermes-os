import {
  ownerAliases,
  pathTokens,
  violation,
} from './validation-diagnostics.mjs';
import { sqlReferencedIdentifiers } from './sql-inspection.mjs';

function tableName(identifier) {
  return identifier.replaceAll('"', '').toLowerCase().split('.').at(-1);
}

function hasOwnerPrefix(identifier, owner) {
  const name = tableName(identifier);
  return [...ownerAliases(owner)].some((alias) => name === alias || name.startsWith(`${alias}_`));
}

function knownOwners(policy) {
  return new Set([
    ...policy.domains.registered,
    ...policy.projections.blockedOwners,
    ...policy.owners.platform,
    ...policy.owners.api,
    policy.owners.core,
  ]);
}

function referencedOwner(policy, identifier) {
  const firstToken = pathTokens(tableName(identifier))[0];
  return [...knownOwners(policy)].find((owner) => ownerAliases(owner).has(firstToken));
}

export function validateStorageEntries(policy, entries) {
  const violations = [];
  const emitted = new Set();

  function emit(code, location, message) {
    const key = `${code}\0${location}\0${message}`;
    if (!emitted.has(key)) violations.push(violation(code, location, message));
    emitted.add(key);
  }

  for (const entry of entries) {
    if (!entry.packageName || !entry.owner || !entry.surface) {
      emit('orphan_sql', entry.path, 'SQL must belong to a registered workspace package');
      continue;
    }
    if (entry.surface !== 'persistence') {
      emit(
        'sql_outside_persistence',
        entry.path,
        `SQL is forbidden in ${entry.packageName} surface=${entry.surface}`,
      );
      continue;
    }

    for (const identifier of sqlReferencedIdentifiers(entry.content)) {
      if (hasOwnerPrefix(identifier, entry.owner)) continue;
      const otherOwner = referencedOwner(policy, identifier);
      if (otherOwner && otherOwner !== entry.owner) {
        emit(
          'cross_owner_sql',
          entry.path,
          `${entry.owner} persistence cannot access ${identifier} owned by ${otherOwner}`,
        );
      } else {
        emit(
          'unowned_sql_identifier',
          entry.path,
          `${identifier} must use the ${entry.owner}_ owner prefix`,
        );
      }
    }
  }

  return violations;
}
