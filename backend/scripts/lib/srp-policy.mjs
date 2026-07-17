import { violation } from './validation-diagnostics.mjs';

export function validateSrpEntries(policy, entries) {
  const violations = [];
  const maximumFileLines = policy.source.maxProductionSourceLines;
  const maximumFunctionLines = policy.source.maxFunctionLines;
  const generatedSegments = new Set(policy.source.generatedPathSegments);
  for (const entry of entries) {
    if (typeof entry.content !== 'string' || entry.content.length === 0) continue;
    const segments = entry.path.split('/');
    if (segments.some((segment) => generatedSegments.has(segment))) continue;
    const lines = entry.content.split(/\r?\n/u);
    if (lines.at(-1) === '') lines.pop();
    if (lines.length > maximumFileLines) {
      violations.push(violation(
        'srp_file_too_large',
        entry.path,
        `source file has ${lines.length} lines; maximum is ${maximumFileLines}`,
      ));
    }
    for (const functionRange of namedFunctionRanges(lines)) {
      if (functionRange.lines > maximumFunctionLines) {
        violations.push(violation(
          'srp_function_too_large',
          `${entry.path}:${functionRange.start}`,
          `${functionRange.name} has ${functionRange.lines} lines; maximum is ${maximumFunctionLines}`,
        ));
      }
    }
  }
  return violations;
}

function namedFunctionRanges(lines) {
  const ranges = [];
  for (let index = 0; index < lines.length; index += 1) {
    const name = functionName(lines[index]);
    if (!name) continue;
    let depth = 0;
    let opened = false;
    for (let cursor = index; cursor < lines.length; cursor += 1) {
      const counts = braceCounts(lines[cursor]);
      depth += counts.open - counts.close;
      opened ||= counts.open > 0;
      if (opened && depth <= 0) {
        ranges.push({ name, start: index + 1, lines: cursor - index + 1 });
        index = cursor;
        break;
      }
    }
  }
  return ranges;
}

function functionName(line) {
  const rust = /^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][\w]*)/u.exec(line);
  if (rust) return rust[1];
  const javascript = /^\s*(?:export\s+)?(?:async\s+)?function\s+([A-Za-z_$][\w$]*)/u.exec(line);
  if (javascript) return javascript[1];
  const arrow = /^\s*(?:export\s+)?const\s+([A-Za-z_$][\w$]*)\s*=.*=>\s*\{/u.exec(line);
  return arrow?.[1];
}

function braceCounts(line) {
  let open = 0;
  let close = 0;
  let quote = null;
  let escaped = false;
  for (let index = 0; index < line.length; index += 1) {
    const character = line[index];
    if (escaped) { escaped = false; continue; }
    if (quote && character === '\\') { escaped = true; continue; }
    if (character === quote) { quote = null; continue; }
    if (!quote && ['"', "'", '`'].includes(character)) { quote = character; continue; }
    if (!quote && character === '/' && line[index + 1] === '/') break;
    if (!quote && character === '{') open += 1;
    if (!quote && character === '}') close += 1;
  }
  return { open, close };
}
