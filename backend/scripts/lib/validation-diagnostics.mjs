export function violation(code, location, message) {
  return { code, location, message };
}

export function list(value) {
  return Array.isArray(value) ? value : [];
}

export function duplicates(values) {
  const seen = new Set();
  const repeated = new Set();
  for (const value of values) {
    if (seen.has(value)) repeated.add(value);
    seen.add(value);
  }
  return [...repeated];
}

export function pathTokens(path) {
  return path.toLowerCase().split(/[\\/._-]+/u).filter(Boolean);
}

export function ownerAliases(owner) {
  const aliases = new Set([owner]);
  if (owner.endsWith('ies')) aliases.add(`${owner.slice(0, -3)}y`);
  else if (owner.endsWith('s')) aliases.add(owner.slice(0, -1));
  else aliases.add(`${owner}s`);
  return aliases;
}

export function formatViolations(violations) {
  return violations
    .map(({ code, location, message }) => `- [${code}] ${location}: ${message}`)
    .join('\n');
}
